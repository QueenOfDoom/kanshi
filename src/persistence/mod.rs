use rusqlite::{Connection, OptionalExtension, Result};

pub fn connect_db() -> Result<Connection> {
    let conn = Connection::open("kanshi.db")?;
    Ok(conn)
}

pub fn initialize_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message (
                  id INTEGER PRIMARY KEY,
                  author INTEGER NOT NULL,
                  content TEXT NOT NULL
         )",
        [],
    )?;
    Ok(())
}

pub fn insert_message(conn: &Connection, id: u64, author: u64, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO message (id, author, content) VALUES (?1, ?2, ?3)",
        &[&id.to_string(), &author.to_string(), content],
    )?;
    Ok(())
}

pub fn update_message_by_id(conn: &Connection, id: u64, new_content: &str) -> Result<()> {
    conn.execute(
        "UPDATE message SET content = ?1 WHERE id = ?2",
        &[new_content, &id.to_string()],
    )?;
    Ok(())
}

pub fn get_message_by_id(conn: &Connection, id: u64) -> Result<Option<(u64, String)>> {
    let mut stmt = conn.prepare("SELECT author, content FROM message WHERE id = ?1")?;
    let result = stmt.query_row(&[&id], |row| {
        let author: u64 = row.get(0)?;
        let content: String = row.get(1)?;
        Ok((author, content))
    }).optional()?;
    Ok(result)
}

pub fn get_message_count(conn: &Connection) -> Result<u64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM message")?;
    let count: u64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}