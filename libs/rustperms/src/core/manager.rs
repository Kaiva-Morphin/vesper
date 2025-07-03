use std::collections::HashMap;
use crate::prelude::*;
use ::tokio::sync::RwLock;



#[derive(Debug)]
pub struct AsyncManager {
    pub users: RwLock<HashMap<Username, User>>,
    pub groups: RwLock<HashMap<Groupname, Group>>,
}
impl Default for AsyncManager {
    fn default() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        }
    }
}

