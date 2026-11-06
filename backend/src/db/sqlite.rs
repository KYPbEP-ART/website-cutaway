use r2d2::{ManageConnection, Pool};
use rusqlite::Connection;
use std::sync::LazyLock;

// 👇 ДОБАВЛЕНО 'pub', чтобы тип был виден в других модулях (auth.rs, users.rs)
pub struct SqliteConnectionManager;

impl ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Connection::open("database.db")
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // Простая проверка, что соединение живо
        conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

// Тип нашего пула (теперь он корректно ссылается на публичный тип)
pub type DbPool = Pool<SqliteConnectionManager>;

// Глобальный пул. Инициализируется один раз при первом обращении.
pub static DB_POOL: LazyLock<DbPool> = LazyLock::new(|| {
    Pool::builder()
        .max_size(10) // 10 одновременных подключений (для SQLite этого более чем достаточно)
        .build(SqliteConnectionManager)
        .expect("Failed to create database pool")
});

pub fn init_db() -> Result<(), rusqlite::Error> {
    // Берем одно соединение из пула для инициализации таблиц
    let conn = DB_POOL.get().expect("Failed to get connection from pool");
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            login TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            avatar TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    tracing::info!("✅ База данных инициализирована (с пулом соединений r2d2)");
    Ok(())
}