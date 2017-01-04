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

extern crate habitat_builder_dbcache as dbcache;
extern crate habitat_builder_protocol as protocol;
extern crate habitat_core as hab_core;
extern crate habitat_net as hab_net;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate protobuf;
extern crate redis;
extern crate serde;
extern crate time;
extern crate toml;
#[macro_use]
extern crate zmq;

pub mod config;
pub mod data_store;
pub mod error;
pub mod server;

pub use self::config::Config;
pub use self::error::{Error, Result};

mod types {
    include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));
}
