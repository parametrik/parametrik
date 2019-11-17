use chrono::{Duration, Utc};
use jsonwebtoken::errors::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::users::models::User;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
}

impl From<&User> for Claims {
    fn from(user: &User) -> Self {
        let now = Utc::now().timestamp();
        Claims {
            sub: user.email.to_string(),
            iat: now,
            exp: now + Duration::days(90).num_seconds(),
        }
    }
}

pub fn create_token(secret: &[u8], user: &User) -> Result<String, Error> {
    let claims = Claims::from(user);
    encode(&Header::default(), &claims, secret)
}

pub fn decode_token(secret: &[u8], token: &str) -> Result<String, Error> {
    let validation = Validation {
        leeway: 60,
        ..Validation::default()
    };
    decode::<Claims>(token, secret, &validation).map(|data| data.claims.sub)
}
