use crate::messages::{AppendEntries, AppendEntriesReply, RequestVote, RequestVoteReply};

#[derive(Debug)]
pub enum RequestError {
    // TODO: Request Error values
    Generic,
}

pub trait Server {
    // TODO: Any way to make the call interface polymorphic? (Is it needed/worthy?)
    fn call_request_vote(
        &self,
        peer: i32,
        req: &RequestVote,
    ) -> Result<RequestVoteReply, RequestError>;

    fn call_append_entries(
        &self,
        peer: i32,
        req: &AppendEntries,
    ) -> Result<AppendEntriesReply, RequestError>;
}
