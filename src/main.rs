use dotenvy::dotenv;
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

#[derive(Deserialize, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    room: String,
    username: String,
    message: String,
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
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

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    // let db = db::establish_db().await;

    rocket::build()
        .attach(cors::CORS)
        .manage(channel::<Message>(1024).0)
        // .manage(states::DBState { db })
        .mount("/", routes![post_msg, events, all_options])
}
