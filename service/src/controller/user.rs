use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::prelude::User as UserEntity;
use service_entity::user;

use crate::errors::ErrorResponder;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User<'r> {
    pub id: i32,
    pub username: &'r str,
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

    pub async fn verified_login(&self, db: &DatabaseConnection) -> bool {
        let already = UserEntity::find()
            .filter(user::Column::Username.eq(self.username.clone()))
            .one(db)
            .await;

        match already {
            Ok(r) => match r {
                Some(_) => false,
                None => true,
            },
            Err(_) => false,
        }
    }

    pub async fn insert(&self, db: &DatabaseConnection) -> Result<(), ErrorResponder> {
        let verified = self.verify_password();
        match verified {
            true => {
                let available = self.verified_login(db).await;
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
                            .exec(db)
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
