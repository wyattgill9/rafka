// crates/broker/src/response.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    // Additional fields if necessary
}

impl Response {
    pub fn new(status_code: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    // Additional methods if necessary
}
