use std::fmt;
use std::fmt::Formatter;

use chrono::Utc;
use jsonwebtoken::{Algorithm, decode, DecodingKey, EncodingKey, Header, Validation};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use warp::{Filter, header, reject, Rejection};
use warp::http::{HeaderMap, HeaderValue};
use warp::http::header::AUTHORIZATION;

use common::data::User;

use crate::error::Error;
use crate::error::Error::JWTTokenCreationError;

const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";

#[derive(Clone, PartialEq)]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn from_str(str: &str) -> Role {
        match str {
            "Admin" => Role::Admin,
            _ => Role::User
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Admin => write!(f, "Admin")
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: i32,
    role: String,
    exp: usize,
}

pub fn create_token(user: &User) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(600))
        .expect("valid timestamp")
        .timestamp();
    let claims = Claims {
        sub: user.id,
        role: Role::Admin.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| JWTTokenCreationError)
}

pub fn with_auth(role: Role) -> impl Filter<Extract=(String, ), Error=Rejection> + Clone {
    header::headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (role.clone(), headers))
        .and_then(authorize)
}

async fn authorize((role, headers): (Role, HeaderMap<HeaderValue>)) -> WebResult<String> {
    match jwt_from_header(&headers) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(Algorithm::HS512),
            )
                .map_err(|_| reject::custom(Error::JWTTokenError))?;

            if role == Role::Admin && Role::from_str(&decoded.claims.role) != Role::Admin {
                return Err(reject::custom(Error::NoPermissionError));
            }

            if role == Role::User && (Role::from_str(&decoded.claims.role) != Role::User ||
                Role::from_str(&decoded.claims.role) != Role::Admin) {
                return Err(reject::custom(Error::NoPermissionError));
            }

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(reject::custom(e)),
    }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Error> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}