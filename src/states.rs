use sea_orm::DatabaseConnection;
pub struct DBState {
    pub db: DatabaseConnection,
}
