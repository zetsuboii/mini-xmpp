use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::session::Session;

/// Struct to represent the state of the server
#[derive(Default, Debug)]
pub struct ServerState {
    /// The connections to the server
    pub sessions: HashMap<String, Arc<Mutex<Session>>>,
}
