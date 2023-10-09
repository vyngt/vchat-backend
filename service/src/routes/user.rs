use rocket::fairing::AdHoc;
use rocket::State;

use super::super::controller::user::{CreateUser, User};
use crate::errors::ErrorResponder;
use crate::states::DBState;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::{json, Json, Value};

#[post("/create", format = "application/json", data = "<new_user>")]
async fn create<'a>(
    new_user: Json<CreateUser<'a>>,
    db: &State<DBState>,
) -> Result<(), ErrorResponder> {
    new_user.insert(&db.conn).await
}

#[get("/me", format = "application/json")]
fn me(user: User, cookies: &CookieJar<'_>) -> Json<Value> {
    cookies.add_private(Cookie::new("user", user.username.clone()));
    Json(json!({
        "name": user.username,
    }))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("User Stage", |rocket| async {
        rocket.mount("/users", routes![create, me])
    })
}
