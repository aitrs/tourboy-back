use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub const DEFAULT_SIZE: i32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginator {
    pub page: i32,
    pub size: i32,
    #[serde(rename = "pageCount")]
    pub page_count: Option<i32>,
    #[serde(rename = "itemCount")]
    pub item_count: Option<i32>,
}

impl Default for Paginator {
    fn default() -> Self {
        Paginator {
            page: 0,
            size: DEFAULT_SIZE,
            page_count: None,
            item_count: None,
        }
    }
}

impl Display for Paginator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " OFFSET {} LIMIT {} ", self.page * self.size, self.size)
    }
}
