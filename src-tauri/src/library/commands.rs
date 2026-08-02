use tauri::State;
use crate::db::DbState;

#[tauri::command]
pub fn add_folder(path: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO folders (path) VALUES (?1)",
        rusqlite::params![path],
    ).map_err(|e| e.to_string())?;
    log::info!("Added folder: {}", path);
    Ok(())
}

#[tauri::command]
pub fn remove_folder(path: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM folders WHERE path = ?1",
        rusqlite::params![path],
    ).map_err(|e| e.to_string())?;
    log::info!("Removed folder: {}", path);
    Ok(())
}

#[tauri::command]
pub fn get_folders(state: State<DbState>) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT path FROM folders")
        .map_err(|e| e.to_string())?;
    let folders = stmt.query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(folders)
}

#[tauri::command]
pub fn get_tracks(state: State<DbState>) -> Result<Vec<crate::library::Track>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, album_artist, duration_secs, track_number, disc_number, year, genre, has_cover FROM tracks ORDER BY artist, album, track_number"
    ).map_err(|e| e.to_string())?;
    let tracks = stmt.query_map([], |row| {
        Ok(crate::library::Track {
            id: row.get(0)?,
            path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            album_artist: row.get(5)?,
            duration_secs: row.get(6)?,
            track_number: row.get(7)?,
            disc_number: row.get(8)?,
            year: row.get(9)?,
            genre: row.get(10)?,
            has_cover: row.get(11)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(tracks)
}

#[tauri::command]
pub fn scan_library(state: State<DbState>) -> Result<(), String> {
    let _scan_lock = state.1.lock().map_err(|e| e.to_string())?;
    let folders: Vec<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT path FROM folders")
            .map_err(|e| e.to_string())?;
        let result: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        result
    };

    for folder in folders {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        super::scan_folder(&conn, &folder);
    }
    Ok(())
}