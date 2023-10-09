use jsonwebtoken::errors::Error;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;

use crate::errors::ErrorResponder;
use crate::states::DBState;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::prelude::User as UserEntity;
use service_entity::user;

use super::jwt::{decode_token, Claims};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl User {
    pub async fn from_jwt_claims(claims: Claims, conn: &DatabaseConnection) -> Option<Self> {
        if claims.refresh {
            return None;
        }
        match Self::get(claims.sub, conn).await {
            Some(record) => Some(Self {
                id: record.id,
                username: record.username,
            }),
            None => None,
        }
    }

    pub async fn get(user_id: i32, conn: &DatabaseConnection) -> Option<user::Model> {
        match UserEntity::find()
            .filter(user::Column::Id.eq(user_id))
            .one(conn)
            .await
        {
            Ok(rc) => rc,
            Err(_) => None,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateUser<'r> {
    pub username: &'r str,
    pub password: &'r str,
    pub password_2: &'r str,
}

impl CreateUser<'_> {
    pub fn verify_password(&self) -> bool {
        self.password.eq(self.password_2)
    }

    pub async fn verified_login(&self, conn: &DatabaseConnection) -> bool {
        let already = UserEntity::find()
            .filter(user::Column::Username.eq(self.username.clone()))
            .one(conn)
            .await;

        match already {
            Ok(r) => match r {
                Some(_) => false,
                None => true,
            },
            Err(_) => false,
        }
    }

    pub async fn insert(&self, conn: &DatabaseConnection) -> Result<(), ErrorResponder> {
        let verified = self.verify_password();
        match verified {
            true => {
                let available = self.verified_login(conn).await;
                match available {
                    true => {
                        let salt = SaltString::generate(&mut OsRng);
                        let argon2 = Argon2::default();

                        let hash_password = argon2
                            .hash_password(self.password.as_bytes(), &salt)
                            .unwrap()
                            .to_string();

                        let new_user = user::ActiveModel {
                            username: ActiveValue::Set(self.username.to_string()),
                            password: ActiveValue::Set(hash_password),
                            ..Default::default()
                        };

                        let _ = UserEntity::insert(new_user)
                            .exec(conn)
                            .await
                            .map_err(|_| ErrorResponder::internal_error("Something went wrong!!!"));

                        Ok(())
                    }
                    false => Err(ErrorResponder::bad_request("Username already exists!!")),
                }
            }
            false => Err(ErrorResponder::bad_request("Password not match!")),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ErrorResponder;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> Result<Claims, Error> {
            Ok(decode_token(String::from(key))?)
        }

        match req.headers().get_one("authorization") {
            None => Outcome::Failure((
                Status::Unauthorized,
                ErrorResponder::unauthorize("No Token provided!!!"),
            )),
            Some(key) => match is_valid(key) {
                Ok(claims) => match req.guard::<&State<DBState>>().await {
                    Outcome::Success(e) => match User::from_jwt_claims(claims, &e.conn).await {
                        Some(u) => Outcome::Success(u),
                        None => Outcome::Failure((
                            Status::Unauthorized,
                            ErrorResponder::unauthorize("Invalid Token!!!"),
                        )),
                    },
                    Outcome::Failure(_) => Outcome::Failure((
                        Status::InternalServerError,
                        ErrorResponder::internal_error("Something went wrong"),
                    )),
                    Outcome::Forward(_) => Outcome::Failure((
                        Status::InternalServerError,
                        ErrorResponder::internal_error("Something went wrong"),
                    )),
                },
                Err(err) => match &err.kind() {
                    _ => Outcome::Failure((
                        Status::Unauthorized,
                        ErrorResponder::unauthorize(&format!("Error Token - {}", err)),
                    )),
                },
            },
        }
    }
}
