// Invoked by candidates to gather votes
#[derive(Debug)]
pub struct RequestVote {
    pub term: i32,          // candidate's term
    pub candidate_id: i32,  // candidate that requests the vote
    pub last_log_idx: i32,  // index of candidate's last log entry
    pub last_log_term: i32, // term of candidate's last log entry
}

#[derive(Debug)]
pub struct RequestVoteReply {
    pub term: i32,          // current term, used by candidate to update itself
    pub vote_granted: bool, // true if the candidate received a vote
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub cmd: Vec<u8>, // TODO: Alternatively could be Box<dyn Any + Send>
    pub term: i32,
}

// Invoked by leader to replicate log entries or perform a heartbeat.
#[derive(Debug)]
pub struct AppendEntries<'a> {
    pub term: i32,               // leader's term
    pub leader_id: i32,          // allows followers to redirect clients
    pub prev_log_idx: i32,       // index of log entry immediately preceding new ones
    pub prev_log_term: i32,      // the term for the last_log_idx
    pub entries: &'a [LogEntry], // empty for heartbeat; may send multiple for efficiency
    pub leader_commit_idx: i32,  // leader's commit index
}

#[derive(Debug)]
pub struct AppendEntriesReply {
    pub term: i32,     // current term, leader uses it to update itself
    pub success: bool, // true if follower contains an entry matching prev_log_idx + prev_log_term
}
