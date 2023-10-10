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
    if let Ok(claims) = decode_token(refresh_body.refresh_token) {
        if claims.refresh {
            if let Some(u) = User::get(claims.sub, &db.conn).await {
                let token = JWTToken::create_token(u.id)?;
                return Ok(Json(token));
            }
        }
    }
    Err(error_resp)
}

#[post("/logout", format = "application/json")]
async fn logout() {}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.mount("/auth", routes![token, logout, refresh])
    })
}
