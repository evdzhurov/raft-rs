use crate::messages::{AppendEntries, AppendEntriesReply, RequestVote, RequestVoteReply};

#[tarpc::service]
pub trait RaftRpc {
    async fn request_vote(req: RequestVote) -> RequestVoteReply;
    async fn append_entries(req: AppendEntries) -> AppendEntriesReply;
}
