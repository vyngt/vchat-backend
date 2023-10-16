use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use service_entity::prelude::Room as RoomEntity;
use service_entity::room;

use crate::errors::ErrorResponder;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Room<'a> {
    pub name: &'a str,
}

impl Room<'_> {
    pub async fn all(conn: &DatabaseConnection) -> Result<Vec<room::Model>, ErrorResponder> {
        match RoomEntity::find()
            .order_by(room::Column::Id, Order::Asc)
            .all(conn)
            .await
        {
            Ok(records) => Ok(records),
            Err(_) => Err(ErrorResponder::internal_error("Something went wrong!!!")),
        }
    }

    pub async fn insert(&self, conn: &DatabaseConnection) -> Result<room::Model, ErrorResponder> {
        let new_room = room::ActiveModel {
            name: ActiveValue::Set(self.name.to_string()),
            ..Default::default()
        };
        if let Ok(rc) = RoomEntity::insert(new_room).exec_with_returning(conn).await {
            Ok(rc)
        } else {
            Err(ErrorResponder::internal_error("Something went wrong!!!"))
        }
    }
}
