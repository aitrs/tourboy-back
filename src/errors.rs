use std::convert::Infallible;

use serde::Serialize;
use thiserror::Error;
use warp::{hyper::StatusCode, reject::Reject, Rejection, Reply};

#[derive(Error, Debug)]
pub enum Error {
    #[error("No Auth Header")]
    NoAuthHeader,
    #[error("Wrong Auth Header")]
    WrongAuthHeader,
    #[error("Unable to get database pool")]
    UnableToGetDatabasePool,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Internal server error")]
    Internal,
    #[error("Database error")]
    Database(String),
    #[error("Authentication error")]
    Auth,
    #[error("Not found")]
    NotFound,
    #[error("misc")]
    Misc,
}

impl Reject for Error {}

#[derive(Serialize, Debug)]
struct ErrorMessage {
    message: String,
    code: String,
}
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::NoAuthHeader | Error::WrongAuthHeader => {
                (StatusCode::UNAUTHORIZED, e.to_string())
            }
            Error::Internal => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Unauthorized | Error::Auth => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::Database(m) => (StatusCode::EXPECTATION_FAILED, m.to_string()),
            Error::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    let json = warp::reply::json(&ErrorMessage {
        code: code.to_string(),
        message: message.clone(),
    });

    eprintln!("Error : {}", message);

    Ok(warp::reply::with_status(json, code))
}
