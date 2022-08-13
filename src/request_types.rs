use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HOIActionData {
    pub bot_name: String,
    pub action: String,
}
