use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub tag: String,
}
