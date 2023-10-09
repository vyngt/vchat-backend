use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::prelude::User as UserEntity;
use service_entity::user;

use crate::errors::ErrorResponder;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Credential<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

impl Credential<'_> {
    fn verify_password(&self, password_hash: &str) -> bool {
        match PasswordHash::new(password_hash) {
            Ok(ph) => Argon2::default()
                .verify_password(self.password.as_bytes(), &ph)
                .is_ok(),
            Err(_) => false,
        }
    }

    pub async fn check_credential(
        &self,
        conn: &DatabaseConnection,
    ) -> Result<user::Model, ErrorResponder> {
        match UserEntity::find()
            .filter(user::Column::Username.eq(self.username.clone()))
            .one(conn)
            .await
        {
            Ok(record) => match record {
                Some(rc) => {
                    if self.verify_password(&rc.password) {
                        Ok(rc)
                    } else {
                        Err(ErrorResponder::unauthorize(
                            "Password or Username not correct!!!",
                        ))
                    }
                }
                None => Err(ErrorResponder::unauthorize(
                    "Password or Username not correct!!!",
                )),
            },
            Err(a) => Err(ErrorResponder::internal_error(a.to_string().as_str())),
        }
    }
}
