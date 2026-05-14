pub mod portability;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub struct Storage {
    pub conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        
        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS procedural_memory (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                strategy TEXT NOT NULL,
                context TEXT,
                success_rate REAL,
                learned_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

pub type SharedStorage = Arc<Storage>;
