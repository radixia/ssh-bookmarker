use crate::error::{AppError, AppResult};
use libsql::{params, Builder, Connection, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub auth_type: String,
    pub key_path: Option<String>,
    pub extra_args: Option<String>,
    pub notes: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: Option<String>,
    #[serde(skip_deserializing)]
    pub updated_at: Option<String>,
}

pub struct Db {
    _database: Database,
    conn: Connection,
}

impl Db {
    pub async fn open(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let database = Builder::new_local(path).build().await?;
        let conn = database.connect()?;
        migrate(&conn).await?;
        Ok(Self { _database: database, conn })
    }

    pub async fn list(&self) -> AppResult<Vec<Bookmark>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, host, user, port, auth_type, key_path, extra_args, notes, created_at, updated_at \
                 FROM bookmarks ORDER BY name COLLATE NOCASE ASC",
                (),
            )
            .await?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(row_to_bookmark(&row)?);
        }
        Ok(out)
    }

    pub async fn get(&self, id: i64) -> AppResult<Bookmark> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, host, user, port, auth_type, key_path, extra_args, notes, created_at, updated_at \
                 FROM bookmarks WHERE id = ?1",
                params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => row_to_bookmark(&row),
            None => Err(AppError::NotFound),
        }
    }

    pub async fn create(&self, b: Bookmark) -> AppResult<Bookmark> {
        self.conn
            .execute(
                "INSERT INTO bookmarks (name, host, user, port, auth_type, key_path, extra_args, notes) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    b.name.clone(),
                    b.host.clone(),
                    b.user.clone(),
                    b.port as i64,
                    b.auth_type.clone(),
                    b.key_path.clone(),
                    b.extra_args.clone(),
                    b.notes.clone(),
                ],
            )
            .await?;
        let id = self.conn.last_insert_rowid();
        self.get(id).await
    }

    pub async fn update(&self, b: Bookmark) -> AppResult<Bookmark> {
        let id = b.id.ok_or_else(|| AppError::Other("missing id".into()))?;
        let affected = self
            .conn
            .execute(
                "UPDATE bookmarks \
                 SET name = ?1, host = ?2, user = ?3, port = ?4, auth_type = ?5, key_path = ?6, \
                     extra_args = ?7, notes = ?8, updated_at = datetime('now') \
                 WHERE id = ?9",
                params![
                    b.name.clone(),
                    b.host.clone(),
                    b.user.clone(),
                    b.port as i64,
                    b.auth_type.clone(),
                    b.key_path.clone(),
                    b.extra_args.clone(),
                    b.notes.clone(),
                    id,
                ],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        self.get(id).await
    }

    pub async fn delete(&self, id: i64) -> AppResult<()> {
        let affected = self
            .conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
            .await?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

async fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            user TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            auth_type TEXT NOT NULL CHECK(auth_type IN ('password','key')),
            key_path TEXT,
            extra_args TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_bookmarks_name ON bookmarks(name);",
    )
    .await?;
    Ok(())
}

fn row_to_bookmark(row: &libsql::Row) -> AppResult<Bookmark> {
    Ok(Bookmark {
        id: Some(row.get::<i64>(0)?),
        name: row.get::<String>(1)?,
        host: row.get::<String>(2)?,
        user: row.get::<String>(3)?,
        port: row.get::<i64>(4)? as u16,
        auth_type: row.get::<String>(5)?,
        key_path: row.get::<Option<String>>(6)?,
        extra_args: row.get::<Option<String>>(7)?,
        notes: row.get::<Option<String>>(8)?,
        created_at: row.get::<Option<String>>(9)?,
        updated_at: row.get::<Option<String>>(10)?,
    })
}
