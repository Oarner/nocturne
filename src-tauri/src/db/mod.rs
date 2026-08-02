use rusqlite::Connection;
use tauri::Manager;
use std::fs;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>, pub Mutex<()>);

pub fn init(app: &tauri::AppHandle) -> Result<DbState, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    fs::create_dir_all(&app_dir)?;
    
    let db_path = app_dir.join("nocturne.db");
    let conn = Connection::open(&db_path)?;
    
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            title TEXT,
            artist TEXT,
            album TEXT,
            album_artist TEXT,
            duration_secs REAL,
            track_number INTEGER,
            disc_number INTEGER,
            year INTEGER,
            genre TEXT,
            has_cover BOOLEAN DEFAULT 0,
            last_modified INTEGER NOT NULL,
            added_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS lyrics (
            track_id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            content TEXT NOT NULL,
            is_synced BOOLEAN DEFAULT 0,
            FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS playlist_tracks (
            playlist_id INTEGER,
            track_id INTEGER,
            position INTEGER NOT NULL,
            PRIMARY KEY(playlist_id, track_id),
            FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
        );
    ")?;

    log::info!("Database initialized at {:?}", db_path);
    Ok(DbState(Mutex::new(conn), Mutex::new(())))
}