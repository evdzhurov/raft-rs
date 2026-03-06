use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};
use tarpc::{client, context};

use crate::consensus::Consensus;
use crate::sm::StateMachine;
use crate::{
    messages::{AppendEntries, AppendEntriesReply, RequestVote, RequestVoteReply},
    rpc::{RaftRpc, RaftRpcClient},
    server::Server,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Peer {
    id: i32,
    addr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterConfig {
    peers: Vec<Peer>,
}

pub struct Node<'a> {
    id: i32,
    peers: HashMap<i32, RaftRpcClient>,
    consensus: Consensus<'a>,
}

impl ClusterConfig {
    pub fn from_file(path: &str) -> Result<ClusterConfig, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: ClusterConfig = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

// TODO: Remove Node, it's a self-referential struct and instead of new can be represented as an entry-point function
impl<'a> Node<'a> {
    pub async fn new(id: i32, config: ClusterConfig, sm: &'a dyn StateMachine) -> Node<'a> {
        let mut peers: HashMap<i32, RaftRpcClient>;
        let mut peer_ids: Vec<i32>;

        for Peer { id: peer_id, addr } in &config.peers {
            if peer_id == &id {
                continue;
            }
            let transport =
                tarpc::serde_transport::tcp::connect(addr, tokio_serde::formats::Json::default)
                    .await
                    .unwrap();
            peers[peer_id] = RaftRpcClient::new(client::Config::default(), transport).spawn();
            peer_ids.push(*peer_id);
        }

        let consensus = Consensus::new(id, peer_ids, sm);

        Self {
            id,
            peers,
            consensus,
        }
    }
}

impl<'a> Server for Node<'a> {
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

impl<'a> RaftRpc for Node<'a> {
    async fn request_vote(self, _: context::Context, req: RequestVote) -> RequestVoteReply {
        todo!();
    }
    async fn append_entries(self, _: context::Context, req: AppendEntries) -> AppendEntriesReply {
        todo!()
    }
}
