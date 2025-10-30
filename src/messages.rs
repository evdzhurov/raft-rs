#[derive(Debug)]
pub struct RequestVote {
    pub term: i32,
    pub candidate_id: i32,
}

#[derive(Debug)]
pub struct RequestVoteReply {
    pub term: i32,
    pub vote_granted: bool,
}

#[derive(Debug)]
pub struct AppendEntries {
    pub term: i32,
    pub leader_id: i32,
}

#[derive(Debug)]
pub struct AppendEntriesReply {
    pub term: i32,
    pub success: bool,
}

impl RequestVote {
    pub fn new(term: i32, candidate_id: i32) -> Self {
        Self { term, candidate_id }
    }
}

impl RequestVoteReply {
    pub fn new(term: i32) -> Self {
        Self {
            term,
            vote_granted: false,
        }
    }
}

impl AppendEntries {
    pub fn new(term: i32, leader_id: i32) -> Self {
        Self { term, leader_id }
    }
}

impl AppendEntriesReply {
    pub fn new(term: i32) -> Self {
        Self {
            term,
            success: false,
        }
    }
}
