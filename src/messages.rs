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
pub struct LogEntry {
    pub cmd: Vec<u8>, // TODO: Alternatively could be Box<dyn Any + Send>
    pub term: i32,
}

#[derive(Debug)]
pub struct AppendEntries<'a> {
    pub term: i32,
    pub leader_id: i32,
    pub last_log_idx: i32,
    pub last_log_term: i32,
    pub leader_commit_idx: i32,
    pub entries: &'a [LogEntry],
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
