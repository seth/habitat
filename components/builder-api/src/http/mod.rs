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

//! A module containing the HTTP server and handlers for servicing client requests

pub mod handlers;
pub mod headers;
mod helpers;

use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use depot;
use hab_net::http::middleware::*;
use hab_net::oauth::github::GitHubClient;
use hab_net::privilege;
use hab_core::event::EventLogger;
use iron::prelude::*;
use mount::Mount;
use persistent::{self, Read};
use staticfile::Static;

use config::Config;
use error::Result;
use self::handlers::*;

// Iron defaults to a threadpool of size `8 * num_cpus`.
// See: http://172.16.2.131:9633/iron/prelude/struct.Iron.html#method.http
const HTTP_THREAD_COUNT: usize = 128;

/// Create a new `iron::Chain` containing a Router and it's required middleware
pub fn router(config: Arc<Config>) -> Result<Chain> {
    let basic = Authenticated::new(&*config);
    let bldr = Authenticated::new(&*config).require(privilege::BUILDER);
    let router = router!(
        status: get "/status" => status,
        authenticate: get "/authenticate/:code" => github_authenticate,
        notify: post "/notify" => notify,

        jobs: post "/jobs" => XHandler::new(job_create).before(bldr.clone()),
        job: get "/jobs/:id" => XHandler::new(job_show).before(bldr.clone()),

        user_invitations: get "/user/invitations" => {
            XHandler::new(list_account_invitations).before(basic.clone())
        },
        edit_user_invitation: put "/user/invitations/:invitation_id" => {
            XHandler::new(accept_invitation).before(basic.clone())
        },
        delete_user_invitation: delete "/user/invitations/:invitation_id" => {
            XHandler::new(ignore_invitation).before(basic.clone())
        },
        user_origins: get "/user/origins" => XHandler::new(list_user_origins).before(basic.clone()),

        projects: post "/projects" => XHandler::new(project_create).before(bldr.clone()),
        project: get "/projects/:origin/:name" => XHandler::new(project_show).before(bldr.clone()),
        edit_project: put "/projects/:origin/:name" => {
            XHandler::new(project_update).before(bldr.clone())
        },
        delete_project: delete "/projects/:origin/:name" => {
            XHandler::new(project_delete).before(bldr.clone())
        }
    );
    let mut chain = Chain::new(router);
    chain.link(persistent::Read::<GitHubCli>::both(GitHubClient::new(&*config)));
    chain.link(Read::<EventLog>::both(EventLogger::new("hab-builder-api", config.events_enabled)));

    chain.link_before(RouteBroker);
    chain.link_after(Cors);
    Ok(chain)
}

/// Create a new HTTP listener and run it in a separate thread. This function will block the calling
/// thread until the new listener has successfully started.
///
/// # Errors
///
/// * Depot could not be started
/// * Couldn't create Router or it's middleware
///
/// # Panics
///
/// * Listener crashed during startup
pub fn run(config: Arc<Config>) -> Result<JoinHandle<()>> {
    let (tx, rx) = mpsc::sync_channel(1);

    let addr = config.http_addr.clone();
    let depot = try!(depot::Depot::new(config.depot.clone()));
    let depot_chain = try!(depot::server::router(depot));

    let mut mount = Mount::new();
    if let Some(ref path) = config.ui_root {
        debug!("Mounting UI at filepath {}", path);
        mount.mount("/", Static::new(path));
    }
    let chain = try!(router(config));
    mount.mount("/v1", chain).mount("/v1/depot", depot_chain);

    let handle = thread::Builder::new()
        .name("http-srv".to_string())
        .spawn(move || {
            let mut server = Iron::new(mount);
            server.threads = HTTP_THREAD_COUNT;
            server.http(addr).unwrap();
            tx.send(()).unwrap();
        })
        .unwrap();
    match rx.recv() {
        Ok(()) => Ok(handle),
        Err(e) => panic!("http-srv thread startup error, err={}", e),
    }
}
