use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Peer {
    id: i32,
    addr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClusterConfig {
    peers: Vec<Peer>,
}

#[tarpc::service]
struct RpcClient;

#[tarpc::service]
struct RpcServer;

struct Node {
    id: i32,
    rpc_server: RpcServer,
    rpc_clients: HashMap<i32, RpcClient>,
}

impl Server for Node {
    fn call_request_vote(
        &self,
        peer: i32,
        req: &crate::messages::RequestVote,
    ) -> Result<crate::messages::RequestVoteReply, crate::server::RequestError> {
        todo!()
    }

    fn call_append_entries(
        &self,
        peer: i32,
        req: &crate::messages::AppendEntries,
    ) -> Result<crate::messages::AppendEntriesReply, crate::server::RequestError> {
        todo!()
    }
}
