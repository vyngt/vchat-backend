use rocket::fairing::AdHoc;
use rocket::http::CookieJar;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{error::RecvError, Sender};
use rocket::{Shutdown, State};

use crate::controller::{
    message::{Message, MessageForm},
    user::User,
};
use crate::errors::ErrorResponder;
use crate::states::DBState;

#[get("/channel/<room_id>")]
async fn events(
    room_id: usize,
    queue: &State<Sender<Message>>,
    mut end: Shutdown,
    cookies: &CookieJar<'_>,
) -> EventStream![] {
    let room_id = room_id as i32;
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
                    Ok(msg) => {
                        if msg.room_id == room_id {
                            msg
                        }else {
                            continue
                        }
                    },
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
async fn post_msg<'a>(
    message: Json<MessageForm<'a>>,
    user: User,
    queue: &State<Sender<Message>>,
    db: &State<DBState>,
) -> Result<(), ErrorResponder> {
    match message.insert(&db.conn, &user).await {
        Ok(msg) => {
            let _x = queue.send(msg);
            Ok(())
        }
        Err(er) => Err(er),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Message Stage", |rocket| async {
        rocket.mount("/chat", routes![post_msg, events])
    })
}
