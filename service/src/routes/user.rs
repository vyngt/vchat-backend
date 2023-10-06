use rocket::fairing::AdHoc;
use rocket::State;

use super::super::controller::user::CreateUser;
use crate::errors::ErrorResponder;
use crate::states::DBState;
use rocket::serde::json::Json;

#[post("/create", format = "application/json", data = "<new_user>")]
async fn create<'a>(
    new_user: Json<CreateUser<'a>>,
    conn: &State<DBState>,
) -> Result<(), ErrorResponder> {
    new_user.insert(&conn.db).await
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("User Stage", |rocket| async {
        rocket.mount("/users", routes![create])
    })
}
