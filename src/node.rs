use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::server::Server;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Peer {
    id: i32,
    addr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClusterConfig {
    peers: Vec<Peer>,
}

struct Node {
    id: i32,
}

impl Server for Node {
    fn call_request_vote(
        &self,
        peer: i32,
        req: &crate::rpc::RequestVote,
    ) -> Result<crate::rpc::RequestVoteReply, crate::server::RequestError> {
        todo!()
    }

    fn call_append_entries(
        &self,
        peer: i32,
        req: &crate::rpc::AppendEntries,
    ) -> Result<crate::rpc::AppendEntriesReply, crate::server::RequestError> {
        todo!()
    }
}
