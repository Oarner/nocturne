#![allow(dead_code)]
pub mod commands;

use serde::Serialize;
use rusqlite::Connection;
use walkdir::WalkDir;
use rayon::prelude::*;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_SCAN_CONCURRENCY: usize = 5;
const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "m4a", "wav", "aac", "opus"];

#[derive(Serialize)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration_secs: Option<f64>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub has_cover: bool,
}

pub struct TrackMeta {
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration_secs: Option<f64>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub has_cover: bool,
    pub last_modified: i64,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn get_last_modified(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or(0)
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn parse_track(path: &Path) -> Option<TrackMeta> {
    let tagged = Probe::open(path).ok()?.read().ok()?;
    let properties = tagged.properties();
    let tag = tagged.primary_tag();

    let duration_secs = Some(properties.duration().as_secs_f64());
    let last_modified = get_last_modified(path);
    let has_cover: bool = tag.map(|t| !t.pictures().is_empty()).unwrap_or(false);

    Some(TrackMeta {
        path: path.to_string_lossy().to_string(),
        title: tag.and_then(|t| t.title().map(|s| s.to_string())),
        artist: tag.and_then(|t| t.artist().map(|s| s.to_string())),
        album: tag.and_then(|t| t.album().map(|s| s.to_string())),
        album_artist: tag.and_then(|t| t.get_string(&lofty::tag::ItemKey::AlbumArtist).map(|s| s.to_string())),
        duration_secs,
        track_number: tag.and_then(|t| t.track()),
        disc_number: tag.and_then(|t| t.disk()),
        year: tag.and_then(|t| t.year().map(|y| y as i32)),
        genre: tag.and_then(|t| t.genre().map(|s| s.to_string())),
        has_cover,
        last_modified,
    })
}

pub fn scan_folder(conn: &Connection, folder_path: &str) {
    let start = std::time::Instant::now();
    log::info!("Scanning folder: {}", folder_path);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(MAX_SCAN_CONCURRENCY)
        .build()
        .expect("Failed to build thread pool");

    // Load existing mtimes from DB
    let existing: std::collections::HashMap<String, i64> = {
        let mut stmt = conn.prepare("SELECT path, last_modified FROM tracks").unwrap();
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    let files: Vec<_> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_supported(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    log::info!("Found {} audio files in {}", files.len(), folder_path);

    // Only parse files that are new or changed
    let tracks: Vec<TrackMeta> = pool.install(|| {
        files.par_iter()
            .filter_map(|path| {
                let path_str = path.to_string_lossy().to_string();
                let current_mtime = get_last_modified(path);
                if let Some(&stored_mtime) = existing.get(&path_str) {
                    if stored_mtime == current_mtime {
                        return None; // unchanged, skip
                    }
                }
                parse_track(path)
            })
            .collect()
    });

    let skipped = files.len() - tracks.len();
    log::info!("Parsed {} new/changed tracks, skipped {} unchanged", tracks.len(), skipped);

    let now = unix_now();
    for track in tracks {
        conn.execute(
            "INSERT OR REPLACE INTO tracks 
            (path, title, artist, album, album_artist, duration_secs, track_number, disc_number, year, genre, has_cover, last_modified, added_at)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            rusqlite::params![
                track.path, track.title, track.artist, track.album,
                track.album_artist, track.duration_secs, track.track_number,
                track.disc_number, track.year, track.genre, track.has_cover,
                track.last_modified, now
            ],
        ).ok();
    }

    log::info!("Scan complete for {} in {}ms", folder_path, start.elapsed().as_millis());
}

pub fn check_deletions(conn: &Connection) {
    let paths: Vec<(i64, String)> = conn
        .prepare("SELECT id, path FROM tracks")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let deleted: Vec<i64> = paths
        .par_iter()
        .filter(|(_, path)| !Path::new(path).exists())
        .map(|(id, _)| *id)
        .collect();

    for id in &deleted {
        conn.execute("DELETE FROM tracks WHERE id = ?1", rusqlite::params![id]).ok();
    }

    if !deleted.is_empty() {
        log::info!("Pruned {} missing tracks", deleted.len());
    }
}