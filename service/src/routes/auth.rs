use super::super::controller::auth::Credential;
use crate::controller::jwt::JWTToken;
use crate::errors::ErrorResponder;
use crate::states::DBState;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;

use rocket::State;

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

#[post("/logout", format = "application/json")]
async fn logout() {}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.mount("/auth", routes![token, logout])
    })
}
