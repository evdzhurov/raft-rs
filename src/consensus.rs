use crate::{
    rpc::{AppendEntries, AppendEntriesReply, LogEntry, RequestVote, RequestVoteReply},
    server::{RequestError, Server},
};
use log::{error, info, warn};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// Raft Sub-problems:
// * Leader Election
// * Log Replication
// * State Machine Safety

// Safety Properties:
// ---------------------------------------------------------------------------------
// * Election Safety - at most one leader can be elected in a given term.
// ---------------------------------------------------------------------------------
// * Leader Append-Only - a leader never overwrites or deletes entries in its log.
// -------------------------------------------------------------------------------
// * Log Matching - if two logs contain an entry with the same index and term, then
// the two logs are identical in all entries up through the given index.
// ---------------------------------------------------------------------------------
// * Leader Completeness - if a log entry is committed in a given term then that
// entry will be present in the logs of the leaders for all higher-numbered terms.
// ---------------------------------------------------------------------------------
// * State Machine Safety - if a server has applied a log entry at a given index
// to its state machine, no other server will ever apply a different log entry
// for the same index.
// ---------------------------------------------------------------------------------

// Server States:
// Candidate -> periodically runs an election and gathers votes to become a leader.
// Leader -> periodically sends heartbeats to followers.
// Follower -> periodically runs an election if it doesn't hear from a leader.
// Dead -> used for graceful shutdown, stops any periodic tasks.

#[derive(Debug, PartialEq, Eq)]
enum State {
    Candidate,
    Leader,
    Follower,
    Dead,
}

#[derive(Debug)]
struct CommitEntry {
    cmd: Vec<u8>,
    index: i32,
    term: i32,
}

// TODO: CommitEntry's command needs to be send back to the caller after reaching consensus
// so it can be applied to the replicated state
// - Use callback function?

struct Consensus<'a, S: Server + ?Sized> {
    server: &'a mut S,
    id: i32,
    peer_ids: Vec<i32>,
    current_term: i32,
    state: State,
    election_reset_event: Instant,
    voted_for: i32,
    commit_idx: i32,
    log: Vec<LogEntry>,
    peer_next_idx: HashMap<i32, i32>,
}

impl<'a, S: Server + ?Sized> Consensus<'a, S> {
    // TODO: new

    fn submit(&mut self, cmd: Vec<u8>) -> bool {
        // TODO: Locking
        if self.state == State::Leader {
            self.log.push(LogEntry {
                cmd: cmd,
                term: self.current_term,
            });
            return true;
        }

        false
    }

    fn start_election(&mut self) {
        // Start election by voting for self
        self.state = State::Candidate;
        self.current_term += 1;

        let saved_current_term = self.current_term;
        self.election_reset_event = Instant::now();

        self.voted_for = self.id;

        // TODO: What is the type of self.log?
        info!(
            "{} becomes Candidate (current_term={saved_current_term}); log=??",
            self.id
        );

        let mut votes_received = 1;

        // TODO: last_log_idx and last_log_term are not yet filled in
        let req = RequestVote {
            term: saved_current_term,
            candidate_id: self.id,
            last_log_idx: 0,
            last_log_term: 0,
        };

        // Send RequestVote RPCs to all other servers concurrently
        for peer_id in self.peer_ids.iter().copied() {
            // TODO: Make a concurrent request

            let reply = self.server.call_request_vote(peer_id, &req);

            match reply {
                Ok(reply) => {
                    info!("{} received RequestVoteReply {:?}", self.id, reply);

                    if self.state != State::Candidate {
                        info!(
                            "{} waiting for reply but state is {:?}",
                            self.id, self.state
                        );
                        return;
                    }

                    if reply.term > saved_current_term {
                        info!("{} term out of date in reply", self.id);
                        self.become_follower(reply.term);
                        return;
                    } else if reply.term == saved_current_term {
                        if reply.vote_granted {
                            votes_received += 1;
                            if votes_received * 2 > self.peer_ids.len() + 1 {
                                // Won majority vote
                                info!("{} won election with {} votes", self.id, votes_received);
                                self.start_leader();
                                return;
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("{} received request error {:?}", self.id, err);
                }
            }
        }

        // In case of failed election...
        self.run_election_timer();
    }

    fn run_election_timer(&mut self) {
        // TODO: This function needs to periodically tick
        // and use synchronization when accessing Consensus state
        // - consider concurrent messages changing the state of Consensus

        // TODO: Needs to be a pseudo-random election timeout (e.g. from 150 to 300 ms)
        let timeout_duration: Duration = Duration::from_micros(200); // TODO: Make it configurable

        let term_started = self.current_term;

        // TODO: print timeout_duration
        info!("Election timer started, term={term_started}");

        // TODO: Check this logic with the Raft paper. We became leader?
        if self.state == State::Leader {
            info!("In election timer state={:?}, bailing out.", self.state);
            return;
        }

        if term_started != self.current_term {
            info!(
                "In election timer changed from {} to {}, bailing out.",
                term_started, self.current_term
            );
            return;
        }

        // Start an election if we haven't heard from a leader or haven't voted for
        // someone for the duration of the timeout.

        if self.election_reset_event.elapsed() >= timeout_duration {
            self.start_election()
        }
    }

    fn start_leader(&mut self) {
        self.state = State::Leader;
        info!(
            "{} becomes leader. term={} log=??",
            self.id, self.current_term
        );

        // TODO: Start sending periodic heartbeats to followers every 50ms (configure it)
        // TODO: tick until leader
        self.leader_send_heartbeats();
    }

    fn leader_send_heartbeats(&mut self) {
        // TODO: Synchronization
        let saved_current_term = self.current_term;

        for peer_id in self.peer_ids.iter().copied() {
            info!("{} sending append entries to {}", self.id, peer_id);

            // TODO: Check why we include the 'prev_idx/term'
            // in the AppendEntries request instead of the 'next'
            // that each follower expects
            let next_idx = self.peer_next_idx[&peer_id];
            let prev_idx = next_idx - 1;
            let prev_term = if prev_idx >= 0 {
                self.log[prev_idx as usize].term
            } else {
                -1
            };

            let entries = if prev_idx >= 0 {
                &self.log[prev_idx as usize..]
            } else {
                &self.log[..]
            };

            let req = AppendEntries {
                term: saved_current_term,
                leader_id: self.id,
                prev_log_idx: prev_idx,
                prev_log_term: prev_term,
                entries: entries,
                leader_commit_idx: self.commit_idx,
            };

            let reply = self.server.call_append_entries(peer_id, &req);

            match reply {
                Ok(reply) => {
                    if reply.term > saved_current_term {
                        info!("{} term out of date in heartbeat reply!", self.id);
                        self.become_follower(reply.term);
                        return;
                    };

                    if self.state == State::Leader && saved_current_term == reply.term {
                        *self
                            .peer_next_idx
                            .get_mut(&peer_id)
                            .expect("update existing peer_next_id") =
                            next_idx + entries.len() as i32;

                        // TODO: what is self.match_index used for?

                        // TODO: Check paper for Append Entries fields meaning
                    }
                }
                Err(err) => {
                    error!(
                        "{} received error {:?} for heartbeat request!",
                        self.id, err
                    );
                }
            }
        }
    }

    fn become_follower(&mut self, term: i32) {
        // TODO: Critical region
        info!("{} becomes follower with term={}, log=??", self.id, term);
        self.state = State::Follower;
        self.current_term = term;
        self.voted_for = -1;
        self.election_reset_event = Instant::now();

        // A follower always runs the election timer in the background
        // TODO: Run in background
        self.run_election_timer();
    }

    fn on_request_vote(&mut self, req: &RequestVote) -> Option<RequestVoteReply> {
        if self.state == State::Dead {
            return None;
        }

        info!("{} received request vote {:?}", self.id, req);

        if req.term > self.current_term {
            info!("{} term out of date in request vote!", self.id);
            self.become_follower(req.term);
        }

        let mut reply = RequestVoteReply {
            term: self.current_term,
            vote_granted: false,
        };

        if self.current_term == req.term
            && (self.voted_for == -1 || self.voted_for == req.candidate_id)
        {
            reply.vote_granted = true;
            self.voted_for = req.candidate_id;
            self.election_reset_event = Instant::now();
        }

        info!("{} reply to request vote: {:?}", self.id, reply);

        Some(reply)
    }

    fn on_append_entries(&mut self, req: &AppendEntries) -> Option<AppendEntriesReply> {
        if self.state == State::Dead {
            return None;
        }

        info!("{} received append entries {:?}", self.id, req);

        if req.term > self.current_term {
            info!("{} term out of date in append entries request!", self.id);
            self.become_follower(req.term);
        }

        let mut reply = AppendEntriesReply {
            term: self.current_term,
            success: false,
        };

        if req.term == self.current_term {
            // TODO: Same term but different leaders exist -> this one becomes a follower.
            // But does it guarantee that only one leader exists?
            // If two leaders back up from being a leader does it mean
            // a new election will be forced?
            if self.state != State::Follower {
                self.become_follower(req.term);
            }

            self.election_reset_event = Instant::now();

            // A successful reply requires that prev_log_idx and prev_log_term exist
            // or the request is trivially satisfied when prev_log_idx == -1
            if req.prev_log_idx == -1
                || ((req.prev_log_idx as usize) < self.log.len()
                    && req.prev_log_term == self.log[req.prev_log_idx as usize].term)
            {
                reply.success = true;

                // Find the insertion point for new log entries - where
                // there's a term mismatch between existing log at prev_log_idx + 1 and
                // new entries in the request

                let mut log_insert_idx = (req.prev_log_idx + 1) as usize;
                let mut new_entries_idx = 0_usize;

                while log_insert_idx < self.log.len()
                    && new_entries_idx < req.entries.len()
                    && self.log[log_insert_idx].term == req.entries[new_entries_idx].term
                {
                    log_insert_idx += 1;
                    new_entries_idx += 1;
                }

                // Postcondition:
                // * log_insert_idx points at the end of the follower's log or an index where
                // there's a mismatch for the term in leader's entry
                // * new_entries_idx points at the end of entries or an index where the term
                // in the follower's log entry does not correspond to the leader's.

                if new_entries_idx < req.entries.len() {
                    self.log.truncate(log_insert_idx); // Keep entries < log_insert_idx
                    self.log.extend_from_slice(&req.entries[new_entries_idx..]);
                }

                if req.leader_commit_idx > self.commit_idx {
                    // TODO: unsigned/signed conversion can cause large values of len to become negative
                    self.commit_idx = req.leader_commit_idx.min(self.log.len() as i32 - 1);
                    info!("{} setting commit index to {}!", self.id, self.commit_idx);

                    // TODO: Signal that new committed entries are ready to be sent up to the client
                }
            }
        }

        info!("{} append entries reply: {:?}", self.id, reply);

        Some(reply)
    }
}
