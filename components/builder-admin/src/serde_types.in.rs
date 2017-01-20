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

use std::net::SocketAddr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Public listening net address for HTTP requests
    pub http_addr: SocketAddr,
    /// List of net addresses for routing servers to connect to
    pub routers: Vec<SocketAddr>,
    /// URL to GitHub API
    pub github_url: String,
    /// Client identifier used for GitHub API requests
    pub github_client_id: String,
    /// Client secret used for GitHub API requests
    pub github_client_secret: String,
    /// Path to UI files to host over HTTP. If not set the UI will be disabled.
    pub ui_root: Option<String>,
}

pub mod http {
    use hab_net::privilege;

    #[derive(Clone, Serialize, Deserialize)]
    pub struct FeatureGrant {
        pub team_id: u64,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct FeatureFlagList(Vec<FeatureFlag>);

    impl Default for FeatureFlagList {
        fn default() -> Self {
            let mut list = vec![];
            list.push(FeatureFlag::new("Admin", privilege::ADMIN.bits()));
            list.push(FeatureFlag::new("Builder", privilege::BUILDER.bits()));
            FeatureFlagList(list)
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct FeatureFlag {
        name: String,
        id: u32,
    }

    impl FeatureFlag {
        pub fn new(name: &'static str, id: u32) -> Self {
            FeatureFlag {
                name: name.to_string(),
                id: id,
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct SearchTerm {
        pub attr: String,
        pub entity: String,
        pub value: String,
    }
}
