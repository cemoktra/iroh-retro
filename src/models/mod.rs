use std::collections::HashSet;

use iroh::EndpointId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RetroNoteItem {
    pub id: String,
    pub content: String,
    pub author: String,
    pub votes: u32,
    pub voted_peers: HashSet<EndpointId>,
}
