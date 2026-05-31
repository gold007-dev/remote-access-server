use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::serde::{Serialize, json::Json};
use rocket::{
    Data, Request,
    data::{self, FromData, ToByteUnit},
    http::Status,
    request::{FromRequest, Outcome},
    response::{Responder, content},
};
use saphyr::{LoadableYamlNode, Yaml};
use serde::Deserialize;
use std::{fmt, fs};
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
}

#[derive(Debug)]
pub enum Error {
    TooLarge,
    NoColon,
    InvalidData,
    Io(std::io::Error),
}
#[rocket::async_trait]
impl<'r> FromData<'r> for User {
    type Error = Error;
    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = req.limits().get("user").unwrap_or(256.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return data::Outcome::Error((Status::PayloadTooLarge, Error::TooLarge)),
            Err(e) => return data::Outcome::Error((Status::InternalServerError, Error::Io(e))),
        };

        let yaml = Yaml::load_from_str(&string).unwrap();
        let body = &yaml[0];
        let some_name = body["name"].as_str();
        let some_password = body["password"].as_str();

        if some_name.is_none() {
            return data::Outcome::Error((Status::UnprocessableEntity, Error::InvalidData));
        }
        let name = some_name.unwrap();
        if some_password.is_none() {
            return data::Outcome::Error((Status::UnprocessableEntity, Error::InvalidData));
        }
        let password = some_password.unwrap();
        let users = get_users();
        let some_id = users.iter().find(|u| {
            u.name == name
                && Argon2::default()
                    .verify_password(
                        password.as_bytes(),
                        &PasswordHash::new(&u.password).unwrap(),
                    )
                    .is_ok()
        });
        if some_id.is_none() {
            return data::Outcome::Error((Status::Forbidden, Error::InvalidData));
        }

        let id = some_id.unwrap().id;

        data::Outcome::Success(User {
            id: id,
            name: name.to_owned(),
            password: password.to_owned(),
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Auth {
    pub user: User,
    pub token: Token,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub user_id: i64,
    pub raw: String,
}

pub struct Tokens {
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct AuthError;

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, _request: &'_ Request<'_>) -> rocket::response::Result<'static> {
        Err(Status::InternalServerError)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The authentication failed")
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = AuthError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, AuthError> {
        let some_token = req.cookies().get("token");
        if some_token.is_none() {
            return Outcome::Error((Status::Unauthorized, AuthError));
        }
        let raw_token = some_token.unwrap().value();

        let token_data = match decode::<Claims>(
            &raw_token,
            &DecodingKey::from_secret(SECRET),
            &Validation::default(),
        ) {
            Ok(data) => data,
            Err(e) => return Outcome::Error((Status::Unauthorized, AuthError)),
        };
        let token = Token {
            raw: raw_token.to_owned(),
            user_id: token_data.claims.user_id,
        };

        let users = get_users();

        let some_user = users.iter().find(|u| token_data.claims.user_id == u.id);
        if some_user.is_none() {
            return Outcome::Error((Status::Unauthorized, AuthError));
        }
        let user = some_user.unwrap().to_owned();

        Outcome::Success(Auth { user, token })
    }
}

pub fn get_users() -> Vec<User> {
    let mut yaml_string = fs::read_to_string("./users.yaml").expect("./users.yaml does not exist");
    if yaml_string.starts_with("\u{feff}") {
        yaml_string = yaml_string
            .strip_prefix("\u{feff}")
            .expect("failed to strip bom")
            .to_owned();
    }

    let settings = Yaml::load_from_str(&yaml_string).unwrap();
    let users_yaml = &settings[0]["users"].as_vec().expect("users not a list");
    let users: Vec<User> = users_yaml
        .iter()
        .map(|user| {
            let name = user["name"].as_str().expect("name not set").to_string();
            let password = user["password"]
                .as_str()
                .expect("password not set")
                .to_string();
            let id = user["id"].as_integer().expect("id not set");
            User { id, name, password }
        })
        .collect();
    return users;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at
}

pub const SECRET: &[u8] = b"your-very-secret-key-change-in-production";

pub fn generate_token(user_id: i64) -> String {
    let now = Utc::now();
    let expire = now + Duration::hours(24); // token valid for 24h
    let claims = Claims {
        user_id,
        exp: expire.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap()
}
