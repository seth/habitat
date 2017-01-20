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

pub mod handlers;

use std::ops::Deref;
use std::sync::{Arc, RwLock};

use protocol::net;
use zmq;

use dbcache::data_store::Pool;
use hab_net::{Application, Supervisor};
use hab_net::config::RouteAddrs;
use hab_net::dispatcher::prelude::*;
use hab_net::routing::Broker;
use hab_net::server::{Envelope, NetIdent, RouteConn, Service, ZMQ_CONTEXT};

use config::Config;
use data_store::DataStore;
use error::{Error, Result};

const BE_LISTEN_ADDR: &'static str = "inproc://backend";

#[derive(Clone)]
pub struct ServerState {
    datastore: Arc<Box<DataStore>>,
}

impl ServerState {
    pub fn new(datastore: DataStore) -> Self {
        ServerState { datastore: Arc::new(Box::new(datastore)) }
    }
}

impl DispatcherState for ServerState {
    fn is_initialized(&self) -> bool {
        true
    }
}

pub struct Worker {
    #[allow(dead_code)]
    config: Arc<RwLock<Config>>,
}

impl Dispatcher for Worker {
    type Config = Config;
    type Error = Error;
    type InitState = ServerState;
    type State = ServerState;

    fn message_queue() -> &'static str {
        BE_LISTEN_ADDR
    }

    fn dispatch(message: &mut Envelope,
                sock: &mut zmq::Socket,
                state: &mut ServerState)
                -> Result<()> {
        let func = match message.message_id() {
            "AccountInvitationListRequest" => handlers::account_invitation_list,
            "CheckOriginAccessRequest" => handlers::origin_check_access,
            "OriginCreate" => handlers::origin_create,
            "OriginGet" => handlers::origin_get,
            "OriginInvitationAcceptRequest" => handlers::origin_invitation_accept,
            "OriginInvitationCreate" => handlers::origin_invitation_create,
            "OriginInvitationListRequest" => handlers::origin_invitation_list,
            "OriginList" => handlers::origin_list,
            "OriginMemberListRequest" => handlers::origin_member_list,
            "AccountOriginListRequest" => handlers::account_origin_list,
            "OriginSecretKeyCreate" => handlers::origin_secret_key_create,
            "OriginSecretKeyGet" => handlers::origin_secret_key_get,
            "ProjectAssociate" => handlers::project_associate,
            "ProjectCreate" => handlers::project_create,
            "ProjectDelete" => handlers::project_delete,
            "ProjectDisassociate" => handlers::project_disassociate,
            "ProjectGet" => handlers::project_get,
            "ProjectUpdate" => handlers::project_update,
            "ProjectSearch" => handlers::project_search,
            "ProjectStateSet" => handlers::project_state_set,
            _ => {
                debug!("dispatch: unhandled message: {}", message.message_id());
                return Ok(());
            }
        };
        func(message, sock, state)
    }

    fn new(config: Arc<RwLock<Config>>) -> Self {
        Worker { config: config }
    }

    fn context(&mut self) -> &mut zmq::Context {
        (**ZMQ_CONTEXT).as_mut()
    }
}

pub struct Server {
    config: Arc<RwLock<Config>>,
    router: RouteConn,
    be_sock: zmq::Socket,
}

impl Server {
    pub fn new(config: Config) -> Result<Self> {
        // JW break; how do we pass a mutable context from static ref?
        let router = try!(RouteConn::new(Self::net_ident(), (**ZMQ_CONTEXT).as_mut()));
        let be = try!((**ZMQ_CONTEXT).as_mut().socket(zmq::DEALER));
        Ok(Server {
            config: Arc::new(RwLock::new(config)),
            router: router,
            be_sock: be,
        })
    }

    pub fn reconfigure(&self, config: Config) -> Result<()> {
        {
            let mut cfg = self.config.write().unwrap();
            *cfg = config;
        }
        // obtain lock and replace our config
        // notify datastore to refresh it's connection if it needs to
        // notify sockets to reconnect if changes
        Ok(())
    }
}

impl Application for Server {
    type Error = Error;

    fn run(&mut self) -> Result<()> {
        try!(self.be_sock.bind(BE_LISTEN_ADDR));
        let datastore = {
            let cfg = self.config.read().unwrap();
            DataStore::start(cfg.deref())
        };
        let cfg = self.config.clone();
        let init_state = ServerState::new(datastore);
        let sup: Supervisor<Worker> = Supervisor::new(cfg, init_state);
        try!(sup.start());
        try!(self.connect());
        let broker = {
            let cfg = self.config.read().unwrap();
            Broker::run(Self::net_ident(), cfg.route_addrs())
        };
        try!(zmq::proxy(&mut self.router.socket, &mut self.be_sock));
        broker.join().unwrap();
        Ok(())
    }
}

impl Service for Server {
    type Application = Self;
    type Config = Config;
    type Error = Error;

    fn protocol() -> net::Protocol {
        net::Protocol::VaultSrv
    }

    fn config(&self) -> &Arc<RwLock<Self::Config>> {
        &self.config
    }

    fn conn(&self) -> &RouteConn {
        &self.router
    }

    fn conn_mut(&mut self) -> &mut RouteConn {
        &mut self.router
    }
}

impl NetIdent for Server {}

pub fn run(config: Config) -> Result<()> {
    try!(Server::new(config)).run()
}
