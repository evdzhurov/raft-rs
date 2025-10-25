pub struct RequestVote {
    term: i32,
    candidate_id: i32,
}

#[derive(Debug)]
pub struct RequestVoteReply {
    pub term: i32,
    pub vote_granted: bool,
}

impl RequestVote {
    pub fn new(term: i32, candidate_id: i32) -> Self {
        Self { term, candidate_id }
    }
}
