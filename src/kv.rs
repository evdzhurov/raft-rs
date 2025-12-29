use std::collections::HashMap;

use crate::sm::StateMachine;

pub struct KvStore {
    dict: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    pub fn get_value(&self, key: &String) -> Option<String> {
        todo!();
        None
    }

    pub fn set_value(&self, key: &String) {
        todo!();
    }

    pub fn del_key(&self, key: &String) {
        todo!();
    }
}

impl StateMachine for KvStore {
    fn apply_command(&self, cmd: Vec<u8>) {
        todo!()
    }
}
