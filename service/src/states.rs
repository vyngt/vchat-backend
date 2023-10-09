use sea_orm::DatabaseConnection;
pub struct DBState {
    pub conn: DatabaseConnection,
}
