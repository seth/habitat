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

pub mod controller;
pub mod headers;
pub mod middleware;
pub mod rendering;

use iron::status::Status;
use protocol::net::ErrCode;

pub fn net_err_to_http(err: ErrCode) -> Status {
    match err {
<<<<<<< 4908cbe5686af955a6d0e415a4b459c0d54ba187
        ErrCode::BUG => Status::InternalServerError,
        ErrCode::TIMEOUT => Status::GatewayTimeout,
        ErrCode::REMOTE_REJECTED => Status::NotAcceptable,
        ErrCode::BAD_REMOTE_REPLY => Status::BadGateway,
        ErrCode::ENTITY_NOT_FOUND => Status::NotFound,
        ErrCode::NO_SHARD => Status::ServiceUnavailable,
        ErrCode::ACCESS_DENIED => Status::Unauthorized,
        ErrCode::SESSION_EXPIRED => Status::Unauthorized,
        ErrCode::ENTITY_CONFLICT => Status::Conflict,
        ErrCode::ZMQ => Status::ServiceUnavailable,
        ErrCode::DATA_STORE => Status::ServiceUnavailable,
        ErrCode::AUTH_SCOPE => Status::Forbidden,
        ErrCode::WORKSPACE_SETUP => Status::InternalServerError,
        ErrCode::SECRET_KEY_FETCH => Status::BadGateway,
        ErrCode::SECRET_KEY_IMPORT => Status::InternalServerError,
        ErrCode::VCS_CLONE => Status::BadGateway,
        ErrCode::BUILD => Status::InternalServerError,
        ErrCode::POST_PROCESSOR => Status::InternalServerError,
=======
        ErrCode::BUG => StatusCode::InternalServerError,
        ErrCode::TIMEOUT => StatusCode::GatewayTimeout,
        ErrCode::REMOTE_REJECTED => StatusCode::NotAcceptable,
        ErrCode::BAD_REMOTE_REPLY => StatusCode::BadGateway,
        ErrCode::ENTITY_NOT_FOUND => StatusCode::NotFound,
        ErrCode::NO_SHARD => StatusCode::ServiceUnavailable,
        ErrCode::ACCESS_DENIED => StatusCode::Unauthorized,
        ErrCode::PROTOCOL_MISMATCH => StatusCode::ServiceUnavailable,
        ErrCode::SESSION_EXPIRED => StatusCode::Unauthorized,
        ErrCode::ENTITY_CONFLICT => StatusCode::Conflict,
        ErrCode::ZMQ => StatusCode::ServiceUnavailable,
        ErrCode::DATA_STORE => StatusCode::ServiceUnavailable,
        ErrCode::AUTH_SCOPE => StatusCode::Forbidden,
        ErrCode::WORKSPACE_SETUP => StatusCode::InternalServerError,
        ErrCode::SECRET_KEY_FETCH => StatusCode::BadGateway,
        ErrCode::SECRET_KEY_IMPORT => StatusCode::InternalServerError,
        ErrCode::VCS_CLONE => StatusCode::BadGateway,
        ErrCode::BUILD => StatusCode::InternalServerError,
        ErrCode::POST_PROCESSOR => StatusCode::InternalServerError,
>>>>>>> Support GitHub push notifications to automatically schedule builds
    }
}
