# Architecture: Node, tokio, tarpc, serde

```mermaid
sequenceDiagram
    participant PeerA as Peer A
    participant transport as serde_transport::tcp
    participant BC as tarpc::BaseChannel
    participant Node as Node (impl RaftRpc)
    participant Consensus
    participant ServerImpl as Node (impl Server)
    participant Client as RaftRpcClient
    participant transport_out as serde_transport::tcp
    participant PeerB as Peer B

    rect rgb(227, 239, 255)
        Note over PeerA, Consensus: Inbound call
        PeerA  ->>  transport: TCP bytes
        transport ->> BC: deserialize (serde)
        BC ->> Node: dispatch request_vote / append_entries
        Node ->> Consensus: on_request_vote / on_append_entries
        Consensus -->> Node: reply
        Node -->> BC: reply
        BC -->> transport: serialize (serde)
        transport -->> PeerA: TCP bytes
    end

    rect rgb(207, 245, 222)
        Note over Consensus, PeerB: Outbound call
        Consensus ->> ServerImpl: call_request_vote / call_append_entries
        ServerImpl ->> Client: .request_vote() / .append_entries()
        Client ->> transport_out: serialize (serde)
        transport_out ->> PeerB: TCP bytes
        PeerB -->> transport_out: TCP bytes
        transport_out -->> Client: deserialize (serde)
        Client -->> ServerImpl: reply
        ServerImpl -->> Consensus: reply
    end
```

## Flow for an inbound call (another node calls us)

1. TCP bytes arrive
2. `serde_transport` deserializes with serde
3. tarpc `BaseChannel` matches request ID
4. Dispatches to `Node::request_vote`
5. Calls `Consensus::on_request_vote`
6. Reply serialized back

## Flow for an outbound call (we call a peer)

1. `Consensus` calls `Node::call_request_vote`
2. `RaftRpcClient::request_vote` called
3. tarpc serializes with serde
4. Sends over TCP
5. Awaits reply future driven by tokio

## Layering summary

```
Node logic
  └── tarpc  (request/reply multiplexing, context, generated client/server)
        └── serde_transport  (frame + serialize/deserialize)
              └── tokio::net::TcpStream  (async I/O)
                    └── tokio runtime  (drives all futures)
```
