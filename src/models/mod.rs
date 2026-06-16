use std::collections::HashSet;

use iroh::EndpointId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RetroNoteItem {
    pub id: String,
    pub content: String,
    pub author: String,
    pub author_id: EndpointId,
    pub votes: u32,
    pub voted_peers: HashSet<EndpointId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    pub id: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimerState {
    pub duration_seconds: u64,
    pub remaining_seconds: u64,
    pub running_since_ms: Option<f64>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            duration_seconds: 300,
            remaining_seconds: 300,
            running_since_ms: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SessionState {
    pub good_notes: Vec<RetroNoteItem>,
    pub bad_notes: Vec<RetroNoteItem>,
    pub action_items: Vec<ActionItem>,
    pub timer: TimerState,
}
