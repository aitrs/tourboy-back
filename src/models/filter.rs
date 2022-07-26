use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(untagged)]
pub enum FilterType {
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "string")]
    String,
}

impl From<String> for FilterType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "numeric" => Self::Numeric,
            "string" => Self::String,
            _ => Self::String,
        }
    }
}

impl Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FilterType::String => "string",
                FilterType::Numeric => "numeric",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(untagged)]
pub enum FilterOp {
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "exact")]
    Exact,
}

impl From<String> for FilterOp {
    fn from(s: String) -> Self {
        match s.as_str() {
            "like" => Self::Like,
            "exact" => Self::Exact,
            _ => Self::Exact,
        }
    }
}

impl FilterOp {
    pub fn to_req<T: Display>(
        &self,
        key: String,
        alias: Option<String>,
        value: T,
        t: FilterType,
    ) -> String {
        let enclose = match t {
            FilterType::Numeric => "",
            FilterType::String => "'",
        };

        let field = if let Some(al) = alias {
            format!("{}.{}", al, key)
        } else {
            key
        };

        match self {
            FilterOp::Exact => format!("{} = {}{}{}", field, enclose, value, enclose),
            FilterOp::Like => format!("{} LIKE '%{}%'", field, value),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filter {
    key: String,
    alias: Option<String>,
    op: FilterOp,
    #[serde(rename = "filterType")]
    filter_type: FilterType,
    value: String,
}

impl Filter {
    pub fn gen_request_append(&self) -> String {
        self.op.to_req(
            self.key.clone(),
            self.alias.clone(),
            self.value.clone(),
            self.filter_type,
        )
    }
}

pub fn gen_request_search(filters: Vec<Filter>) -> String {
    filters
        .iter()
        .map(|f| f.gen_request_append())
        .collect::<Vec<String>>()
        .join(" AND ")
}
