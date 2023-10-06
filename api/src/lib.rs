use dotenvy::dotenv;
use migration::MigratorTrait;
use rocket::http::{Cookie, CookieJar};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Shutdown, State};

#[macro_use]
extern crate rocket;

mod cors;
mod db;
mod states;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Credential {
    username: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ResponseJson {
    message: String,
}

#[derive(Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    room: String,
    username: String,
    message: String,
}

#[get("/events")]
async fn events(
    queue: &State<Sender<Message>>,
    mut end: Shutdown,
    cookies: &CookieJar<'_>,
) -> EventStream![] {
    match cookies.get_private("user") {
        Some(e) => {
            println!("{:?}", e);
        }
        None => {}
    }

    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

#[post("/message", data = "<message>")]
fn post_msg(message: Json<Message>, queue: &State<Sender<Message>>) {
    let _x = queue.send(message.into_inner());
}

#[post("/login", data = "<credential>")]
fn login(credential: Json<Credential>, cookies: &CookieJar<'_>) -> Json<ResponseJson> {
    let crd = credential.into_inner();
    cookies.add_private(Cookie::new("user", crd.username));

    Json(ResponseJson {
        message: "success".into(),
    })
}

/// I hate you!!!!!
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[launch]
pub async fn rocket() -> _ {
    dotenv().ok();

    let db = db::establish_db().await;

    // Run Migrations
    migration::Migrator::up(&db, None).await.unwrap();

    rocket::build()
        .attach(cors::CORS)
        .manage(channel::<Message>(1024).0)
        .manage(states::DBState { db })
        .mount("/", routes![post_msg, events, all_options, login])
}
