use rocket::fairing::AdHoc;
use rocket::State;

use crate::controller::{room::Room, user::User};

use crate::errors::ErrorResponder;
use crate::states::DBState;
use rocket::serde::json::Json;

use service_entity::room;

#[get("/")]
async fn get_all_rooms(
    _user: User,
    db: &State<DBState>,
) -> Result<Json<Vec<room::Model>>, ErrorResponder> {
    match Room::all(&db.conn).await {
        Ok(rooms) => Ok(Json(rooms)),
        Err(resp) => Err(resp),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Room Stage", |rocket| async {
        rocket.mount("/rooms", routes![get_all_rooms])
    })
}
