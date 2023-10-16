use controller::message::Message;
use dotenvy::dotenv;
use migration::MigratorTrait;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{content, status};

use rocket::tokio::sync::broadcast::channel;

#[macro_use]
extern crate rocket;

mod controller;
mod cors;
mod db;
mod errors;
mod routes;
mod states;

/// I hate you!!!!!
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[catch(default)]
fn default_catcher(
    status: Status,
    _request: &Request<'_>,
) -> status::Custom<content::RawJson<String>> {
    let mut status_reason = "Unknown";
    if let Some(reason) = status.reason() {
        status_reason = reason;
    }

    status::Custom(
        status,
        content::RawJson(format!("{{\"message\": \"{status_reason}\"}}")),
    )
}

#[launch]
pub async fn rocket() -> _ {
    dotenv().ok();

    let conn = db::establish_db().await;

    // Run Migrations
    migration::Migrator::up(&conn, None).await.unwrap();

    rocket::build()
        .manage(channel::<Message>(1024).0)
        .manage(states::DBState { conn })
        .attach(cors::CORS)
        .attach(routes::user::stage())
        .attach(routes::auth::stage())
        .attach(routes::message::stage())
        .attach(routes::room::stage())
        .mount("/", routes![all_options])
        .register("/", catchers![default_catcher])
}
