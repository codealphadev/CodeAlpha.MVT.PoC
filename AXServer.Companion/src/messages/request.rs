use serde::{Deserialize, Serialize};

pub use models::Connect;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Request {
    Connect(Connect),
    None,
}
