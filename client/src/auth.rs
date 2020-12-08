use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAuth {
    pub uid: String,
    pub secret: String,
}