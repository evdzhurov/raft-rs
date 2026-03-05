# raft-rs Project Context

## Overview
Educational Raft consensus protocol implementation in Rust. Work in progress — does not yet compile.
- Rust edition: 2024
- Binary: `node` — accepts `--node-id` and `--config <json file>`

## Dependencies (Cargo.toml)
- `clap 4.5` — CLI arg parsing (derive feature)
- `log 0.4` — logging facade
- `serde 1.0` + `serde_json 1.0` — JSON serialization (cluster config)
- `tarpc 0.37` — async RPC framework (serde-transport + tcp features)

## Source Files
- `src/main.rs` — entry point: parses args, loads ClusterConfig, constructs KvStore/Node/Consensus (stubs)
- `src/messages.rs` — RPC message structs (was rpc.rs in earlier version)
- `src/rpc.rs` — tarpc service trait `RaftRpc` (async, generates client/server via `#[tarpc::service]`)
- `src/server.rs` — synchronous `Server` trait abstracting outbound RPC calls
- `src/consensus.rs` — core Raft logic
- `src/node.rs` — `Node` and `ClusterConfig`; implements both `Server` and `RaftRpc` traits
- `src/sm.rs` — `StateMachine` trait
- `src/kv.rs` — `KvStore` implementing `StateMachine`

## Key Types

### messages.rs
- `RequestVote { term, candidate_id, last_log_idx, last_log_term: i32 }`
- `RequestVoteReply { term: i32, vote_granted: bool }`
- `LogEntry { cmd: Vec<u8>, term: i32 }` (derives Clone)
- `AppendEntries { term, leader_id, prev_log_idx, prev_log_term, entries: Vec<LogEntry>, leader_commit_idx: i32 }` — owned Vec, not a slice
- `AppendEntriesReply { term: i32, success: bool }`

### rpc.rs
- `RaftRpc` tarpc service trait — generates `RaftRpcClient` and server stub
  - `async fn request_vote(req: RequestVote) -> RequestVoteReply`
  - `async fn append_entries(req: AppendEntries) -> AppendEntriesReply`

### server.rs
- `RequestError` enum — only `Generic` variant (TODO: add more)
- `Server` trait (TODO: rename to `RaftNode`?): synchronous outbound calls
  - `call_request_vote(peer: i32, req: &RequestVote) -> Result<RequestVoteReply, RequestError>`
  - `call_append_entries(peer: i32, req: &AppendEntries) -> Result<AppendEntriesReply, RequestError>`

### consensus.rs
- `State` enum: `Candidate`, `Leader`, `Follower`, `Dead`
- `CommitEntry { cmd: Vec<u8>, index: i32, term: i32 }`
- `Consensus<'a>` — no longer generic; uses `&'a mut dyn Server` and `&'a mut dyn StateMachine`
  - Fields: `server, sm, id, peer_ids, current_term, state, election_reset_event, voted_for, commit_idx, log, peer_next_idx`
- Methods: `submit`, `start_election`, `run_election_timer`, `start_leader`, `leader_send_heartbeats`, `become_follower`, `on_request_vote`, `on_append_entries`
- `Consensus::new` not yet implemented

### node.rs
- `Peer { id: i32, addr: String }` (Serialize/Deserialize)
- `ClusterConfig { peers: Vec<Peer> }` — loaded from JSON via `from_file(path)`
- `Node<'a> { id: i32, peers: HashMap<i32, RaftRpcClient>, consensus: Consensus<'a> }`
  - `Node::new` is `async`, connects to peers via tarpc TCP transport
  - Implements `Server` (outbound sync calls) and `RaftRpc` (incoming async handler) — both `todo!()`

### sm.rs
- `StateMachine` trait: `fn apply_command(&self, cmd: Vec<u8>)` (requires `Send`)

### kv.rs
- `KvStore { dict: HashMap<String, String> }` — implements `StateMachine`
- `get_value`, `set_value`, `del_key` — all `todo!()`

## Architecture
```
Client
  └─> Node  (implements Server trait + RaftRpc tarpc service)
        ├─> Consensus  (core Raft state machine)
        │     └─> StateMachine  (e.g. KvStore)
        └─> HashMap<i32, RaftRpcClient>  (tarpc clients, one per peer)
```

## Known TODOs / Gaps
- `Consensus::new` not implemented
- `Node::new` has compile errors (uninitialized `peers`/`peer_ids`, wrong struct init syntax)
- `main` calls `Server::new` and `Consensus::new` with wrong/missing args; no async runtime
- No async runtime (tokio) wired up — `Node::new` is async but `main` is sync
- All peer RPCs still sequential in consensus; needs async concurrency
- Election timeout hardcoded to 200µs; should be randomized 150–300ms
- Heartbeat periodic task (every 50ms) not implemented
- `last_log_idx`/`last_log_term` always 0 in `start_election`
- `match_index` per-peer not tracked (needed for leader commit advancement)
- `CommitEntry` delivery to client not wired up (callback or channel)
- No locking/synchronization in consensus
- `RequestError` has only `Generic` variant
- tarpc message types need `Serialize`/`Deserialize` derives (not yet added)
- `leader_send_heartbeats` sends `log[prev_idx..]` — likely should be `log[next_idx..]`
- Possible signed/unsigned overflow: `self.log.len() as i32` for large logs

## References
- https://raft.github.io/
- Eli Bendersky's Raft series: Parts 0, 1, 2 (Elections + Log Replication)
