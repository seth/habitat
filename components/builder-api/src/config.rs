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

//! Configuration for a Habitat Builder-API service

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use hab_net;
use hab_net::config::{DEFAULT_GITHUB_URL, DEV_GITHUB_CLIENT_ID, DEV_GITHUB_CLIENT_SECRET,
                      GitHubOAuth, RouteAddrs};
use hab_core::config::{ConfigFile, ParseInto};
use depot;
use toml;

pub use types::Config;
use error::{Error, Result};

impl Config {
    /// Set the port of the http listener
    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.http_addr.set_port(port);
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            http_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9636)),
            routers: vec![SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5562))],
            depot: depot::Config::default(),
            github_url: DEFAULT_GITHUB_URL.to_string(),
            github_client_id: DEV_GITHUB_CLIENT_ID.to_string(),
            github_client_secret: DEV_GITHUB_CLIENT_SECRET.to_string(),
            ui_root: None,
            events_enabled: false, // TODO: change to default to true later
        }
    }
}

impl ConfigFile for Config {
    type Error = Error;

    fn from_toml(toml: toml::Value) -> Result<Self> {
        let mut cfg = Config::default();
        let mut pkg_path = String::new();
        if try!(toml.parse_into("pkg.svc_static_path", &mut pkg_path)) {
            cfg.ui_root = Some(pkg_path);
        }
        try!(toml.parse_into("cfg.http_addr", &mut cfg.http_addr));
        try!(toml.parse_into("cfg.router_addrs", &mut cfg.routers));
        try!(toml.parse_into("pkg.svc_data_path", &mut cfg.depot.path));
        try!(toml.parse_into("cfg.depot.datastore_addr", &mut cfg.depot.datastore_addr));
        try!(toml.parse_into("cfg.github.url", &mut cfg.github_url));
        try!(toml.parse_into("cfg.github.url", &mut cfg.depot.github_url));
        try!(toml.parse_into("cfg.github.client_id", &mut cfg.github_client_id));
        if cfg.github_client_id.is_empty() {
            return Err(Error::from(hab_net::Error::RequiredConfigField("github.client_id")));
        }
        try!(toml.parse_into("cfg.github.client_id", &mut cfg.depot.github_client_id));
        try!(toml.parse_into("cfg.github.client_secret", &mut cfg.github_client_secret));
        if cfg.github_client_secret.is_empty() {
            return Err(Error::from(hab_net::Error::RequiredConfigField("github.client_secret")));
        }
        try!(toml.parse_into("cfg.github.client_secret",
                             &mut cfg.depot.github_client_secret));
        try!(toml.parse_into("cfg.events_enabled", &mut cfg.events_enabled));
        try!(toml.parse_into("cfg.events_enabled", &mut cfg.depot.events_enabled));
        Ok(cfg)
    }
}

impl RouteAddrs for Config {
    fn route_addrs(&self) -> &Vec<SocketAddr> {
        &self.routers
    }
}

impl GitHubOAuth for Config {
    fn github_url(&self) -> &str {
        &self.github_url
    }

    fn github_client_id(&self) -> &str {
        &self.github_client_id
    }

    fn github_client_secret(&self) -> &str {
        &self.github_client_secret
    }
}
