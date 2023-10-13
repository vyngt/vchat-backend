use super::user::User;
use crate::errors::ErrorResponder;
use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::message;
use service_entity::prelude::Message as MessageEntity;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    id: i32,
    content: String,
    pub user: User,
    pub room_id: i32,
    created_at: chrono::NaiveDateTime,
}

impl Message {
    pub fn from_data(msg: message::Model, user: User) -> Self {
        Self {
            id: msg.id,
            content: msg.content,
            user,
            room_id: msg.room_id,
            created_at: msg.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct MessageForm<'a> {
    content: &'a str,
    room_id: i32,
}

impl MessageForm<'_> {
    pub async fn insert(
        &self,
        conn: &DatabaseConnection,
        user: &User,
    ) -> Result<Message, ErrorResponder> {
        let new_message = message::ActiveModel {
            content: ActiveValue::Set(self.content.to_string()),
            user_id: ActiveValue::Set(user.id),
            room_id: ActiveValue::Set(self.room_id),
            ..Default::default()
        };
        if let Ok(rc) = MessageEntity::insert(new_message)
            .exec_with_returning(conn)
            .await
        {
            Ok(Message::from_data(rc, user.clone()))
        } else {
            Err(ErrorResponder::internal_error("Something went wrong!!!"))
        }
    }
}