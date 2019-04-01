use rusqlite::{params, Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("foo.db")?;

    conn.execute("CREATE TABLE roo", params![])?;

    Ok(())
}
