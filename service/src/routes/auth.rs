use super::super::controller::auth::Credential;
use crate::controller::jwt::{decode_token, JWTToken};
use crate::controller::user::User;
use crate::errors::ErrorResponder;
use crate::states::DBState;
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket::State;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RefreshBody<'a> {
    refresh_token: &'a str,
}

#[post("/token", format = "application/json", data = "<credential>")]
async fn token<'a>(
    credential: Json<Credential<'a>>,
    db: &State<DBState>,
) -> Result<Json<JWTToken>, ErrorResponder> {
    match credential.check_credential(&db.conn).await {
        Ok(user) => {
            let token = JWTToken::create_token(user.id)?;
            Ok(Json(token))
        }
        Err(resp) => Err(resp),
    }
}

#[post("/refresh", format = "application/json", data = "<refresh_body>")]
async fn refresh<'a>(
    refresh_body: Json<RefreshBody<'a>>,
    db: &State<DBState>,
) -> Result<Json<JWTToken>, ErrorResponder> {
    // TODO deactivate previous refresh token
    let error_resp = ErrorResponder::bad_request("Invalid Token!!!");
    match decode_token(refresh_body.refresh_token) {
        Ok(claims) => {
            if claims.refresh {
                match User::get(claims.sub, &db.conn).await {
                    Some(rc) => {
                        let token = JWTToken::create_token(rc.id)?;
                        Ok(Json(token))
                    }
                    None => Err(error_resp),
                }
            } else {
                Err(error_resp)
            }
        }
        Err(_) => Err(error_resp),
    }
}

#[post("/logout", format = "application/json")]
async fn logout() {}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.mount("/auth", routes![token, logout, refresh])
    })
}
