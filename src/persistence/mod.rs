pub(crate) mod schema;
pub(crate) mod models;

use diesel::{Connection, SqliteConnection};
use diesel::prelude::*;
use crate::persistence::models::*;
use crate::util::UNKNOWN_USER;

pub fn establish_connection() -> SqliteConnection {
    let database_url = std::env::var("DATABASE_URL")
        .expect("Expected a database url in the environment");
    SqliteConnection::establish(&database_url)
        .expect("Database Connection Failure")
}

pub fn get_author_from_message(message_id: u64) -> u64 {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
    messages.find(message_id as i64)
        .select(Message::as_select())
        .first(connection)
        .optional()
        .expect("Error loading messages")
        .map_or(UNKNOWN_USER, |m| m.author as u64)
}

pub fn get_message_content_by_id(message_id: u64) -> String {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
    messages.find(message_id as i64)
        .select(Message::as_select())
        .first(connection)
        .optional()
        .expect("Error loading messages")
        .map_or("<unknown message>".to_string(), |m| m.content.clone())
}

pub fn get_message_content_and_author_by_id(message_id: u64) -> (u64, String) {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
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

pub fn get_message_count() -> i64 {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
    messages.count().get_result::<i64>(connection).expect("Error loading messages.")
}

pub fn create_message(message_id: u64, author_id: u64, text: String) -> Message {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
    let new_message = Message { id: message_id as i64, author: author_id as i64, content: text };
    diesel::insert_into(messages).values(&new_message).execute(connection)
        .expect("Error creating message.");
    new_message
}

pub fn update_message_content(message_id: u64, text: String) {
    use crate::persistence::schema::messages::dsl::*;
    let connection = &mut establish_connection();
    diesel::update(messages.find(message_id as i64))
        .set(content.eq(text))
        .execute(connection).expect("Error updating message.");
}