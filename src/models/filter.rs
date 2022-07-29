use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Copy, Clone)]
pub enum FilterType {
    Numeric,
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

#[derive(Debug, Copy, Clone)]
pub enum FilterOp {
    Like,
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
        like_start: bool,
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
        let prefix = if !like_start {
            "%".to_string()
        } else {
            "".to_string()
        };

        match self {
            FilterOp::Exact => format!("{} = {}{}{}", field, enclose, value, enclose),
            FilterOp::Like => format!("{} LIKE '{}{}%'", field, prefix, value),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterIntermediate {
    key: String,
    alias: Option<String>,
    op: String,
    #[serde(rename = "filterType")]
    filter_type: String,
    value: String,
    #[serde(rename = "likeStart")]
    like_start: bool,
}

#[derive(Debug, Clone)]
pub struct Filter {
    key: String,
    alias: Option<String>,
    op: FilterOp,
    filter_type: FilterType,
    value: String,
    like_start: bool,
}

impl From<FilterIntermediate> for Filter {
    fn from(inter: FilterIntermediate) -> Self {
        Filter {
            key: inter.key,
            alias: inter.alias,
            op: FilterOp::from(inter.op),
            filter_type: FilterType::from(inter.filter_type),
            value: inter.value,
            like_start: inter.like_start,
        }
    }
}

impl Filter {
    pub fn gen_request_append(&self) -> String {
        self.op.to_req(
            self.key.clone(),
            self.alias.clone(),
            self.value.clone(),
            self.filter_type,
            self.like_start,
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
