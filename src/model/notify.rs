use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NotifyProfile {
    pub address: String,
    pub password: String,
    pub smtp_addr: String,
}