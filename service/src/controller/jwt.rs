use crate::errors::ErrorResponder;
use chrono::{Duration, Utc};
use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::serde::{Deserialize, Serialize};
use sha256::digest;
use std::env;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    pub sub: i32,
    pub jti: String,
    pub refresh: bool,
    exp: usize,
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims,
}

pub fn create_token(
    user_id: i32,
    jti: &str,
    refresh: bool,
    duration: Duration,
) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");

    let expiration = Utc::now()
        .checked_add_signed(duration)
        .expect("Invalid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        jti: jti.into(),
        refresh,
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str) -> Result<Claims, ErrorKind> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let token = token.trim_start_matches("Bearer").trim();
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err.kind().to_owned()),
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct JWTToken {
    access_token: String,
    refresh_token: String,
}

/// Get duration time(seconds)
fn get_duration(variable: &str, default: i64) -> i64 {
    match env::var(variable) {
        Ok(s) => {
            let d: i64 = s.parse().unwrap_or(default);
            d
        }
        Err(_) => default,
    }
}

impl JWTToken {
    fn get_token_durations() -> (i64, i64) {
        let access_duration = get_duration("JWT_ACCESS_DURATION", 60);
        let refresh_duration = get_duration("JWT_REFRESH_DURATION", 300);
        (access_duration, refresh_duration)
    }

    fn _create_tokens(user_id: i32) -> Result<Self, Error> {
        let (access, refresh) = Self::get_token_durations();
        let mut input = Utc::now().to_string();
        input.push_str(user_id.to_string().as_str());
        let jti = digest(input);
        let access_token = create_token(user_id, &jti, false, Duration::seconds(access))?;
        let refresh_token = create_token(user_id, &jti, true, Duration::seconds(refresh))?;

        Ok(Self {
            access_token,
            refresh_token,
        })
    }

    pub fn create_token(user_id: i32) -> Result<Self, ErrorResponder> {
        match Self::_create_tokens(user_id) {
            Ok(jwt) => Ok(jwt),
            Err(_) => Err(ErrorResponder::internal_error("Something went wrong!!!")),
        }
    }
}
