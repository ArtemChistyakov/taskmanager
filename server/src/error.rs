use std::convert::Infallible;

use mobc_postgres::tokio_postgres;
use serde_derive::Serialize;
use thiserror::Error;
use warp::{Rejection, Reply};
use warp::http::StatusCode;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error getting connection from DB pool: {0}")]
    DBPoolError(mobc::Error<tokio_postgres::Error>),
    #[error("error executing DB query: {0}")]
    DBQueryError(tokio_postgres::Error),
    #[error("error creating table: {0}")]
    DBInitError(tokio_postgres::Error),
    #[error("error creating table")]
    DBInitErrorTest,
    #[error("error encrypt password: {0}")]
    EncryptPasswordError(bcrypt::BcryptError),
    #[error("error verify password: {0}")]
    VerifyPasswordError(bcrypt::BcryptError),
    #[error("error reading file: {0}")]
    ReadFileError(std::io::Error),
    #[error("wrong credentials")]
    WrongCredentialsError,
    #[error("jwt token not valid")]
    JWTTokenError,
    #[error("jwt token creation error")]
    JWTTokenCreationError,
    #[error("no auth header")]
    NoAuthHeaderError,
    #[error("invalid auth header")]
    InvalidAuthHeaderError,
    #[error("no permission")]
    NoPermissionError,
    #[error("user not activated")]
    UserNotEnabledError,
    #[error("error send notification")]
    NotificationError,
}

impl From<mobc::Error<mobc_postgres::tokio_postgres::Error>> for Error {
    fn from(e: mobc::Error<tokio_postgres::Error>) -> Self {
        Error::DBPoolError(e)
    }
}

impl From<mobc_postgres::tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::DBQueryError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::ReadFileError(e)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::DBQueryError(_) => {
                code = StatusCode::BAD_REQUEST;
                message = "Could not Execute request";
            }
            Error::WrongCredentialsError => {
                code = StatusCode::BAD_REQUEST;
                message = "Credentials not valid.";
            }
            Error::JWTTokenCreationError => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
            Error::NoAuthHeaderError => {
                code = StatusCode::UNAUTHORIZED;
                message = "Unauthorized";
            }
            Error::InvalidAuthHeaderError => {
                code = StatusCode::UNAUTHORIZED;
                message = "Unauthorized";
            }
            Error::NoPermissionError => {
                code = StatusCode::FORBIDDEN;
                message = "Forbidden";
            }
            Error::JWTTokenError => {
                code = StatusCode::UNAUTHORIZED;
                message = "Unauthorized";
            }
            Error::UserNotEnabledError => {
                code = StatusCode::BAD_REQUEST;
                message = "User not enabled"
            }
            _ => {
                eprintln!("unhandled application error: {:?}", err);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ErrorResponse {
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}