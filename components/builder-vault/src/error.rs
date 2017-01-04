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

use std::error;
use std::fmt;
use std::io;
use std::result;

use hab_core;
use hab_net;
use protobuf;
use dbcache;
use zmq;

#[derive(Debug)]
pub enum Error {
    BadGitHubCloneURL(String),
    BadPort(String),
    Config(hab_core::config::ConfigError),
    DataStore(dbcache::Error),
    HabitatCore(hab_core::Error),
    IO(io::Error),
    NetError(hab_net::Error),
    Protobuf(protobuf::ProtobufError),
    UnknownVCS,
    Zmq(zmq::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::BadGitHubCloneURL(ref e) => format!("{} is an invalid GitHub clone url.", e),
            Error::BadPort(ref e) => format!("{} is an invalid port. Valid range 1-65535.", e),
            Error::Config(ref e) => format!("{}", e),
            Error::DataStore(ref e) => format!("DataStore error, {}", e),
            Error::HabitatCore(ref e) => format!("{}", e),
            Error::IO(ref e) => format!("{}", e),
            Error::NetError(ref e) => format!("{}", e),
            Error::Protobuf(ref e) => format!("{}", e),
            Error::UnknownVCS => {
                "Unknown VCS type assigned to project. Protocol mismatch?".to_string()
            }
            Error::Zmq(ref e) => format!("{}", e),
        };
        write!(f, "{}", msg)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadGitHubCloneURL(_) => "VCS contained a bad GitHub clone url.",
            Error::BadPort(_) => "Received an invalid port or a number outside of the valid range.",
            Error::Config(ref err) => err.description(),
            Error::DataStore(ref err) => err.description(),
            Error::HabitatCore(ref err) => err.description(),
            Error::IO(ref err) => err.description(),
            Error::NetError(ref err) => err.description(),
            Error::Protobuf(ref err) => err.description(),
            Error::UnknownVCS => {
                "A project had an unknown VCS type. This is typically due to a protocol mismatch."
            }
            Error::Zmq(ref err) => err.description(),
        }
    }
}

impl From<hab_core::Error> for Error {
    fn from(err: hab_core::Error) -> Error {
        Error::HabitatCore(err)
    }
}

impl From<hab_core::config::ConfigError> for Error {
    fn from(err: hab_core::config::ConfigError) -> Error {
        Error::Config(err)
    }
}

impl From<dbcache::Error> for Error {
    fn from(err: dbcache::Error) -> Self {
        Error::DataStore(err)
    }
}

impl From<hab_net::Error> for Error {
    fn from(err: hab_net::Error) -> Self {
        Error::NetError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(err: protobuf::ProtobufError) -> Self {
        Error::Protobuf(err)
    }
}

impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Error::Zmq(err)
    }
}
