mod consensus;
mod node;
mod rpc;
mod server;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "raft-rs", about = "Raft node")]
struct Args {
    #[arg(long, short)]
    node_id: i32,
    #[arg(long, short)]
    config: String,
}

fn main() {
    let args = Args::parse();
    println!("Starting node {} with config {}", args.node_id, args.config);
}
