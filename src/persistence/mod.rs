pub(crate) mod schema;
pub(crate) mod models;

use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PoolError};
use r2d2::{Error, Pool, PooledConnection};
use crate::persistence::models::*;
use crate::util::UNKNOWN_USER;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> SqlitePool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("Expected a database url in the environment");
    init_pool(&database_url).expect("Failed to create Database Pool")
}

pub fn init_pool(database_url: &str) -> Result<SqlitePool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn sqlite_pool_handler(pool: &SqlitePool) -> Result<SqlitePooledConnection, Error> {
    pool.get()
}

pub fn get_author_from_message(pool: &SqlitePool, message_id: u64) -> u64 {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    messages.find(message_id as i64)
        .select(Message::as_select())
        .first(connection)
        .optional()
        .expect("Error loading messages")
        .map_or(UNKNOWN_USER, |m| m.author as u64)
}

pub fn get_message_content_by_id(pool: &SqlitePool, message_id: u64) -> String {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    messages.find(message_id as i64)
        .select(Message::as_select())
        .first(connection)
        .optional()
        .expect("Error loading messages")
        .map_or("<unknown message>".to_string(), |m| m.content.clone())
}

pub fn get_message_content_and_author_by_id(pool: &SqlitePool, message_id: u64) -> (u64, String) {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    let results = messages.find(message_id as i64)
        .select(Message::as_select())
        .first(connection)
        .optional()
        .expect("Error loading messages");
    
    results.map_or(
        (UNKNOWN_USER, "<unknown_message>".to_string()),
        |m| (m.author as u64, m.content.clone())
    )
}

pub fn exists_message(pool: &SqlitePool, message_id: u64) -> bool {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    messages.find(message_id as i64).count().get_result::<i64>(connection)
        .expect("Error loading messages.") > 0
}

pub fn get_message_count(pool: &SqlitePool) -> i64 {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    messages.count().get_result::<i64>(connection).expect("Error loading messages.")
}

pub fn create_message(pool: &SqlitePool, message_id: u64, author_id: u64, text: String) -> Message {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    let new_message = Message { id: message_id as i64, author: author_id as i64, content: text };
    diesel::insert_into(messages).values(&new_message).execute(connection)
        .expect("Error creating message.");
    new_message
}

pub fn update_message_content(pool: &SqlitePool, message_id: u64, text: String) {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut sqlite_pool_handler(&pool)
        .expect("Pooled Connection failed.");
    diesel::update(messages.find(message_id as i64))
        .set(content.eq(text))
        .execute(connection).expect("Error updating message.");
}