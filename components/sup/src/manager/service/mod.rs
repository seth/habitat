// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod config;

use std;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use ansi_term::Colour::{Yellow, Red, Green};
use hcore::service::ServiceGroup;
use hcore::crypto::hash;
use hcore::fs;
use hcore::util::perm::{set_owner, set_permissions};

pub use types::service::*;
pub use self::config::ServiceConfig;
use config::gconfig;
use error::Result;
use health_check;
use manager::signals;
use manager::census::CensusList;
use package::Package;
use supervisor::{Supervisor, RuntimeConfig};
use util;

static LOGKEY: &'static str = "SR";

impl Service {
    pub fn new<S: Into<String>>(package: Package,
                                group: S,
                                organization: Option<String>,
                                topology: Topology,
                                update_strategy: UpdateStrategy)
                                -> Result<Service> {
        let service_group = ServiceGroup::new(package.name.clone(), group, organization);
        let (svc_user, svc_group) = try!(util::users::get_user_and_group(&package.pkg_install));
        let sg = format!("{}.{}", service_group.service, service_group.group);
        outputln!(preamble sg, "Process will run as user={}, group={}",
                  &svc_user,
                  &svc_group);
        let runtime_config = RuntimeConfig::new(svc_user, svc_group);
        let supervisor = Supervisor::new(package.ident().clone(), &service_group, runtime_config);
        Ok(Service {
            service_group: service_group,
            supervisor: supervisor,
            package: package,
            topology: topology,
            needs_restart: false,
            update_strategy: update_strategy,
            current_service_files: HashMap::new(),
            last_restart_display: LastRestartDisplay::None,
            initialized: false,
            service_config_incarnation: None,
        })
    }

    pub fn service_group_str(&self) -> String {
        format!("{}", self.service_group)
    }

    pub fn start(&mut self) -> Result<()> {
        self.supervisor.start()
    }

    pub fn restart(&mut self, census_list: &CensusList) -> Result<()> {
        match self.topology {
            Topology::Leader | Topology::Initializer => {
                if let Some(census) = census_list.get(&format!("{}.{}",
                                                               self.service_group.service,
                                                               self.service_group.group)) {
                    // We know perfectly well we are in this census, because we asked for
                    // our own service group *by name*
                    let me = census.me().unwrap();
                    if me.get_election_is_running() {
                        if self.last_restart_display != LastRestartDisplay::ElectionInProgress {
                            outputln!(preamble self.service_group_str(),
                                      "Not restarting service; {}",
                                      Yellow.bold().paint("election in progress."));
                            self.last_restart_display = LastRestartDisplay::ElectionInProgress;
                        }
                    } else if me.get_election_is_no_quorum() {
                        if self.last_restart_display != LastRestartDisplay::ElectionNoQuorum {
                            outputln!(preamble self.service_group_str(),
                                      "Not restarting service; {}, {}.",
                                      Yellow.bold().paint("election in progress"),
                                      Red.bold().paint("and we have no quorum"));
                            self.last_restart_display = LastRestartDisplay::ElectionNoQuorum;
                        }
                    } else if me.get_election_is_finished() {
                        // We know we have a leader, so this is fine
                        let leader_id = census.get_leader().unwrap().get_member_id();
                        if self.last_restart_display != LastRestartDisplay::ElectionFinished {
                            outputln!(preamble self.service_group_str(),
                                      "Restarting service; {} is the leader",
                                      Green.bold().paint(leader_id));
                            self.last_restart_display = LastRestartDisplay::ElectionFinished;
                        }
                        self.needs_restart = false;
                        try!(self.supervisor.restart());
                    }
                }
            }
            Topology::Standalone => {
                self.needs_restart = false;
                try!(self.supervisor.restart());
            }
        }
        Ok(())
    }

    pub fn down(&mut self) -> Result<()> {
        self.supervisor.down()
    }

    pub fn send_signal(&self, signal: u32) -> Result<()> {
        match self.supervisor.child {
            Some(ref child) => signals::send_signal(child.id(), signal),
            None => {
                debug!("No process to send the signal to");
                Ok(())
            }
        }
    }

    pub fn is_down(&self) -> bool {
        self.supervisor.child.is_none()
    }

    pub fn check_process(&mut self) -> Result<()> {
        self.supervisor.check_process()
    }

    pub fn write_butterfly_service_file(&mut self,
                                        filename: String,
                                        incarnation: u64,
                                        body: Vec<u8>)
                                        -> bool {
        self.current_service_files.insert(filename.clone(), incarnation);
        let on_disk_path = fs::svc_files_path(&self.service_group.service).join(filename);
        let current_checksum = match hash::hash_file(&on_disk_path) {
            Ok(current_checksum) => current_checksum,
            Err(e) => {
                debug!("Failed to get current checksum for {:?}: {}",
                       on_disk_path,
                       e);
                String::new()
            }
        };
        let new_checksum = hash::hash_bytes(&body)
            .expect("We failed to hash a Vec<u8> in a method that can't return an error; not \
                     even sure what this means");
        if new_checksum != current_checksum {
            let new_filename = format!("{}.write", on_disk_path.to_string_lossy());

            let mut new_file = match File::create(&new_filename) {
                Ok(new_file) => new_file,
                Err(e) => {
                    outputln!(preamble self.service_group_str(),
                        "Service file from butterfly failed to open the new file {}: {}",
                        new_filename,
                        Red.bold().paint(format!("{}", e)));
                    return false;
                }
            };

            if let Err(e) = new_file.write_all(&body) {
                outputln!(preamble self.service_group_str(),
                    "Service file from butterfly failed to write {}: {}",
                    new_filename,
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = std::fs::rename(&new_filename, &on_disk_path) {
                outputln!(preamble self.service_group_str(),
                    "Service file from butterfly failed to rename {} to {}: {}",
                    new_filename,
                    on_disk_path.to_string_lossy(),
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = set_owner(&on_disk_path,
                                      &self.supervisor.runtime_config.svc_user,
                                      &self.supervisor.runtime_config.svc_group) {
                outputln!(preamble self.service_group_str(),
                    "Service file from butterfly failed to set ownership on {}: {}",
                    on_disk_path.to_string_lossy(),
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = set_permissions(&on_disk_path, 0o640) {
                outputln!(preamble self.service_group_str(),
                    "Service file from butterfly failed to set permissions on {}: {}",
                    on_disk_path.to_string_lossy(),
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            outputln!(preamble self.service_group_str(),
                "Service file updated from butterfly {}: {}",
                on_disk_path.to_string_lossy(),
                Green.bold().paint(new_checksum));
            true
        } else {
            false
        }
    }

    pub fn write_butterfly_service_config(&mut self, config: String) -> bool {
        let on_disk_path = fs::svc_path(&self.service_group.service).join("gossip.toml");
        let current_checksum = match hash::hash_file(&on_disk_path) {
            Ok(current_checksum) => current_checksum,
            Err(e) => {
                debug!("Failed to get current checksum for {:?}: {}",
                       on_disk_path,
                       e);
                String::new()
            }
        };
        let new_checksum = hash::hash_string(&config)
            .expect("We failed to hash a string in a method that can't return an error; not even \
                     sure what this means");
        if new_checksum != current_checksum {
            let new_filename = format!("{}.write", on_disk_path.to_string_lossy());

            let mut new_file = match File::create(&new_filename) {
                Ok(new_file) => new_file,
                Err(e) => {
                    outputln!(preamble self.service_group_str(),
                        "Service configuration from butterfly failed to open the new file: {}",
                        Red.bold().paint(format!("{}", e)));
                    return false;
                }
            };

            if let Err(e) = new_file.write_all(config.as_bytes()) {
                outputln!(preamble self.service_group_str(),
                    "Service configuration from butterfly failed to write: {}",
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = std::fs::rename(&new_filename, &on_disk_path) {
                outputln!(preamble self.service_group_str(),
                    "Service configuration from butterfly failed to rename: {}",
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = set_owner(&on_disk_path,
                                      &self.supervisor.runtime_config.svc_user,
                                      &self.supervisor.runtime_config.svc_group) {
                outputln!(preamble self.service_group_str(),
                    "Service configuration from butterfly failed to set ownership: {}",
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            if let Err(e) = set_permissions(&on_disk_path, 0o640) {
                outputln!(preamble self.service_group_str(),
                    "Service configuration from butterfly failed to set permissions: {}",
                    Red.bold().paint(format!("{}", e)));
                return false;
            }

            outputln!(preamble self.service_group_str(),
                "Service configuration updated from butterfly: {}",
                Green.bold().paint(new_checksum));
            true
        } else {
            false
        }
    }

    pub fn health_check(&self) -> Result<health_check::CheckResult> {
        self.package.health_check(&self.supervisor, &self.service_group)
    }

    pub fn file_updated(&self) {
        if self.initialized {
            match self.package.file_updated(&self.service_group) {
                Ok(_) => outputln!(preamble self.service_group_str(), "{}", "File update hook succeeded."),
                Err(e) => {
                    outputln!(preamble self.service_group_str(), "File update hook failed: {}", e)
                }
            }
        }
    }

    pub fn initialize(&mut self) {
        if !self.initialized {
            match self.package.initialize(&self.service_group) {
                Ok(()) => {
                    outputln!(preamble self.service_group_str(), "{}", "Initializing");
                    self.initialized = true
                }
                Err(e) => {
                    outputln!(preamble self.service_group_str(), "Initialization failed: {}", e)
                }
            }
        }
    }

    pub fn reconfigure(&mut self, census_list: &CensusList) {
        let sg = format!("{}", self.service_group);
        let mut service_config =
            match ServiceConfig::new(&sg, &self.package, census_list, gconfig().bind()) {
                Ok(sc) => sc,
                Err(e) => {
                    outputln!(preamble self.service_group_str(),
                              "Error generating Service Configuration; not reconfiguring: {}",
                              e);
                    return;
                }
            };
        match self.package.create_svc_path() {
            Ok(_) => {}
            Err(e) => outputln!("Failed to create the svc path: {}", e),
        }
        match service_config.write(&self.package) {
            Ok(true) => {
                self.needs_restart = true;
                match self.package.reconfigure(&self.service_group) {
                    Ok(_) => {}
                    Err(e) => {
                        outputln!(preamble self.service_group_str(),
                            "Reconfiguration hook failed: {}", e);
                    }
                }
            }
            Ok(false) => {}
            Err(e) => {
                outputln!(preamble self.service_group_str(),
                    "Failed to write service configuration: {}", e);
            }
        }

        self.package.hooks().load_hooks();
        // Probably worth moving the run hook under compile all, eventually
        if let Err(e) = self.package.copy_run(&service_config) {
            outputln!("Failed to copy run hook: {}", e);
        }
        self.package.hooks().compile_all(&service_config);
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.package)
    }
}
