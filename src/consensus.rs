use crate::{
    messages::RequestVote,
    server::{RequestError, Server},
};
use log::{error, info};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
enum State {
    // TODO: Unknown State?
    Follower,
    Candidate,
    Leader,
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

        // Send RequestVote RPCs to all other servers concurrently
        for peer_id in self.peer_ids.iter().copied() {
            // TODO: Make a concurrent request

            let req = RequestVote::new(saved_current_term, self.id);
            let resp = self.server.call_request_vote(peer_id, &req);

            match resp {
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

    fn become_follower(&self, term: i32) {}

    fn start_leader(&self) {}
}
