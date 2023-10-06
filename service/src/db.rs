use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use std::env;

pub async fn create_db_if_not_exists(url: &str) -> String {
    let db_name = env::var("DB_NAME").expect("DB: DB_NAME is not set");

    let db = Database::connect(url).await.unwrap();

    match &db.get_database_backend() {
        sea_orm::DatabaseBackend::Postgres => {
            match db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", db_name),
                ))
                .await
            {
                Ok(_) => format!("{}/{}", url, db_name),
                Err(e) => {
                    let expected = format!("Execution Error: error returned from database: database \"{}\" already exists", db_name);
                    if e.to_string() == expected {
                        format!("{}/{}", url, db_name)
                    } else {
                        String::new()
                    }
                }
            }
        }
        _other => String::new(),
    }
}

pub async fn establish_db() -> DatabaseConnection {
    let db_host = env::var("DB_HOST").expect("DB: DB_HOST is not set");
    let db_port = env::var("DB_PORT").expect("DB: DB_PORT is not set");
    let db_user = env::var("DB_USER").expect("DB: DB_USER is not set");
    let db_password = env::var("DB_PASSWORD").expect("DB: DB_PASSWORD is not set");

    let base_url = format!(
        "postgres://{}:{}@{}:{}",
        db_user, db_password, db_host, db_port
    );
    let database_url = create_db_if_not_exists(&base_url).await;

    let db = Database::connect(database_url).await;
    let connection = db.expect("DB Connection Problem...");
    connection
}
