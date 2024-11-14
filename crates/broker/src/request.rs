// crates/broker/src/request.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    // Additional fields if necessary
}

impl Request {
    pub fn new(headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self { headers, body }
    }

    // Additional methods if necessary
}
