use super::user::User;
use crate::errors::ErrorResponder;
use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::message;
use service_entity::prelude::Message as MessageEntity;
use service_entity::prelude::User as UserEntity;

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

    pub async fn fetch_from_room_id(room_id: i32, conn: &DatabaseConnection) -> Vec<Self> {
        let mut results = vec![];
        if let Ok(messages) = MessageEntity::find()
            .filter(message::Column::RoomId.eq(room_id))
            .order_by_asc(message::Column::CreatedAt)
            .all(conn)
            .await
        {
            if let Ok(users) = messages.load_one(UserEntity, conn).await {
                for (msg_record, user_record) in messages.into_iter().zip(users) {
                    if let Some(u) = user_record {
                        results.push(Self::from_data(
                            msg_record,
                            User {
                                id: u.id,
                                username: u.username,
                            },
                        ));
                    }
                }
            }
        };

        results
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct MessageForm {
    content: String,
    room_id: i32,
}

impl MessageForm {
    pub async fn insert(
        &self,
        conn: &DatabaseConnection,
        user: &User,
    ) -> Result<Message, ErrorResponder> {
        let new_message = message::ActiveModel {
            content: ActiveValue::Set(self.content.clone()),
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
