pub trait StateMachine: Send {
    fn apply_command(&self, cmd: Vec<u8>);
}
