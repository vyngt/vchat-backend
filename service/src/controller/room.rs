use sea_orm::*;
use service_entity::prelude::Room as RoomEntity;
use service_entity::room;

use crate::errors::ErrorResponder;

pub struct Room;

impl Room {
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
}
