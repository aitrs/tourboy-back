use std::fmt::Display;

use errors::Error;

pub mod auth;
pub mod config;
pub mod errors;
pub mod mailer;
pub mod models;
pub mod paginator;
pub mod router;

pub fn db_error_to_warp(e: anyhow::Error) -> crate::Error {
    Error::Database(e.to_string())
}

pub fn etointlog(e: impl Display) -> crate::Error {
    eprintln!("{}", e);
    Error::Internal
}
