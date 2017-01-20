// Copyright:: Copyright (c) 2015-2016 Chef Software, Inc.
//
// The terms of the Evaluation Agreement (Bldr) between Chef Software Inc. and the party accessing
// this file ("Licensee") apply to Licensee's use of the Software until such time that the Software
// is made available under an open source license such as the Apache 2.0 License.

use std::collections::BTreeMap;

use protobuf::{self, Message};
use redis;
use rustc_serialize::json::{Json, ToJson};

use message::{Persistable, Routable};
use sharding::InstaId;

pub use message::vault::*;

impl Persistable for Origin {
    type Key = u64;

    fn primary_key(&self) -> Self::Key {
        self.get_id()
    }

    fn set_primary_key(&mut self, value: Self::Key) {
        self.set_id(value);
    }
}

impl Routable for OriginGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        // JW TODO: This won't accurately find the origin without it. We can switch to using the ID
        // of the origin or perform a reverse lookup by storing the name->ID map on a particular
        // vault server.
        Some(self.get_name().to_string())
    }
}

impl Routable for OriginCreate {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_owner_id()))
    }
}

impl ToJson for Origin {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("id".to_string(), self.get_id().to_string().to_json());
        m.insert("name".to_string(), self.get_name().to_json());
        m.insert("owner_id".to_string(),
                 self.get_owner_id().to_string().to_json());
        m.insert("private_key_name".to_string(),
                 self.get_private_key_name().to_json());
        Json::Object(m)
    }
}

impl Routable for OriginMemberRemove {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_origin_id()))
    }
}

impl Routable for OriginMemberListRequest {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_origin_id()))
    }
}

impl Persistable for OriginSecretKey {
    type Key = u64;

    fn primary_key(&self) -> Self::Key {
        self.get_id()
    }

    fn set_primary_key(&mut self, value: Self::Key) {
        self.set_id(value);
    }
}

impl ToJson for OriginSecretKey {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("id".to_string(), self.get_id().to_string().to_json());
        m.insert("origin_id".to_string(),
                 self.get_origin_id().to_string().to_json());
        m.insert("name".to_string(), self.get_name().to_json());
        m.insert("revision".to_string(),
                 self.get_revision().to_string().to_json());
        m.insert("body".to_string(),
                 String::from_utf8(self.get_body().to_vec()).unwrap().to_json());
        m.insert("owner_id".to_string(),
                 self.get_owner_id().to_string().to_json());
        Json::Object(m)
    }
}

impl Routable for OriginSecretKeyCreate {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_owner_id()))
    }
}

impl Routable for OriginSecretKeyGet {
    type H = InstaId;

    fn route_key(&self) -> Option<Self::H> {
        Some(InstaId(self.get_owner_id()))
    }
}

impl Routable for OriginInvitationCreate {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_owner_id())
    }
}

impl Persistable for OriginInvitation {
    type Key = u64;

    fn primary_key(&self) -> Self::Key {
        self.get_id()
    }

    fn set_primary_key(&mut self, value: Self::Key) {
        self.set_id(value);
    }
}

impl ToJson for OriginInvitation {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("id".to_string(), self.get_id().to_string().to_json());
        m.insert("account_id".to_string(),
                 self.get_account_id().to_string().to_json());
        m.insert("account_name".to_string(),
                 self.get_account_name().to_json());
        m.insert("origin_id".to_string(),
                 self.get_origin_id().to_string().to_json());
        m.insert("origin_name".to_string(), self.get_origin_name().to_json());
        m.insert("owner_id".to_string(),
                 self.get_owner_id().to_string().to_json());
        Json::Object(m)
    }
}

impl Routable for AccountInvitationListRequest {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_account_id())
    }
}

impl Routable for AccountInvitationListResponse {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_account_id())
    }
}

impl ToJson for AccountInvitationListResponse {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("account_id".to_string(),
                 self.get_account_id().to_string().to_json());
        m.insert("invitations".to_string(), self.get_invitations().to_json());
        Json::Object(m)
    }
}

impl Routable for OriginInvitationListRequest {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_origin_id())
    }
}

impl Routable for OriginInvitationListResponse {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_origin_id())
    }
}

impl ToJson for OriginInvitationListResponse {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("origin_id".to_string(),
                 self.get_origin_id().to_string().to_json());
        m.insert("invitations".to_string(), self.get_invitations().to_json());
        Json::Object(m)
    }
}

impl Routable for OriginInvitationAcceptRequest {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        // we don't have an origin id here...
        Some(self.get_invite_id())
    }
}

impl ToJson for OriginMemberListResponse {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("origin_id".to_string(),
                 self.get_origin_id().to_string().to_json());
        m.insert("members".to_string(), self.get_members().to_json());
        Json::Object(m)
    }
}

impl Routable for AccountOriginListRequest {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_account_id())
    }
}

impl ToJson for AccountOriginListResponse {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("account_id".to_string(),
                 self.get_account_id().to_string().to_json());
        m.insert("origins".to_string(), self.get_origins().to_json());
        Json::Object(m)
    }
}

impl Routable for CheckOriginAccessRequest {
    type H = u64;

    fn route_key(&self) -> Option<Self::H> {
        // TODO!
        Some(self.get_account_id())
    }
}

impl Routable for ProjectGet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_id().to_string())
    }
}

impl Routable for ProjectStateSet {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_id().to_string())
    }
}

impl Routable for ProjectAssociate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_term().to_string())
    }
}

impl Routable for ProjectCreate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_project().get_id().to_string())
    }
}

impl Routable for ProjectDelete {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_id().to_string())
    }
}

impl Routable for ProjectDisassociate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_term().to_string())
    }
}

impl Routable for ProjectSearch {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_term().to_string())
    }
}

impl Routable for ProjectUpdate {
    type H = String;

    fn route_key(&self) -> Option<Self::H> {
        Some(self.get_project().get_id().to_string())
    }
}

impl Persistable for Project {
    type Key = String;

    fn primary_key(&self) -> Self::Key {
        self.get_id().to_string()
    }

    fn set_primary_key(&mut self, value: Self::Key) {
        self.set_id(value);
    }
}

impl ToJson for Project {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("id".to_string(), self.get_id().to_json());
        m.insert("plan_path".to_string(),
                 self.get_plan_path().to_string().to_json());
        m.insert("vcs".to_string(), self.get_git().to_json());
        Json::Object(m)
    }
}

impl ToJson for VCSGit {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("type".to_string(), "git".to_string().to_json());
        m.insert("url".to_string(), self.get_url().to_string().to_json());
        Json::Object(m)
    }
}

impl redis::FromRedisValue for Project {
    fn from_redis_value(value: &redis::Value) -> redis::RedisResult<Project> {
        let bytes = try!(Vec::from_redis_value(value));
        Ok(protobuf::parse_from_bytes(&bytes).unwrap())
    }
}

impl redis::ToRedisArgs for Project {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![self.write_to_bytes().unwrap()]
    }
}

impl<'a> redis::ToRedisArgs for &'a Project {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![self.write_to_bytes().unwrap()]
    }
}
