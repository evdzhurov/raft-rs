use crate::{
    messages::{AppendEntries, AppendEntriesReply, RequestVote, RequestVoteReply},
    server::{RequestError, Server},
};
use log::{error, info, warn};
use std::time::{Duration, Instant};

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

struct Consensus<'a, S: Server + ?Sized> {
    server: &'a mut S,
    id: i32,
    peer_ids: Vec<i32>,
    current_term: i32,
    state: State,
    election_reset_event: Instant,
    voted_for: i32,
}

impl<'a, S: Server + ?Sized> Consensus<'a, S> {
    // TODO: new

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

        let req = RequestVote::new(saved_current_term, self.id);

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
        // TODO: Atomic load
        let saved_current_term = self.current_term;

        let req = AppendEntries::new(saved_current_term, self.id);

        for peer_id in self.peer_ids.iter().copied() {
            info!("{} sending append entries to {}", self.id, peer_id);

            let reply = self.server.call_append_entries(peer_id, &req);

            match reply {
                Ok(reply) => {
                    if reply.term > saved_current_term {
                        info!("{} term out of date in heartbeat reply!", self.id);
                        self.become_follower(reply.term);
                        return;
                    };
                }
                Err(err) => {
                    error!("{} received error for heartbeat request!", self.id);
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

        let mut reply = RequestVoteReply::new(self.current_term);

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

        let mut reply = AppendEntriesReply::new(self.current_term);

        if req.term == self.current_term {
            // TODO: Same term but different leaders exist -> this one becomes a follower.
            // But does it guarantee that only one leader exists?
            // If two leaders back up from being a leader does it mean
            // a new election will be forced?
            if self.state != State::Follower {
                self.become_follower(req.term);
            }

            self.election_reset_event = Instant::now();
            reply.success = true;
        }

        info!("{} append entries reply: {:?}", self.id, reply);

        Some(reply)
    }
}
