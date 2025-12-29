mod consensus;
mod kv;
mod messages;
mod node;
mod rpc;
mod server;
mod sm;

use crate::{
    consensus::Consensus,
    kv::KvStore,
    node::{ClusterConfig, Node},
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "node", about = "Raft node")]
struct Args {
    #[arg(long, short)]
    node_id: i32,
    #[arg(long, short)]
    config: String,
}

fn main() {
    let args = Args::parse();
    println!("Starting node {} with config {}", args.node_id, args.config);

    let config = match ClusterConfig::from_file(&args.config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load config {}: {}", args.config, err);
            std::process::exit(1);
        }
    };

    let kv_store = KvStore::new();

    let server = Server::new(args.node_id, config);

    let consensus = Consensus::new(args.node_id);
}
