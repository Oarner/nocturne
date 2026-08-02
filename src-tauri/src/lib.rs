use tauri::Manager;
mod db;
mod library;
mod player;
mod lyrics;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db_state = db::init(app.handle())?;
            app.manage(db_state);
            app.manage(player::Player::new());

           let app_handle = app.handle().clone();
std::thread::spawn(move || {
    let state = app_handle.state::<db::DbState>();
    let _scan_lock = state.1.lock().unwrap();
    let conn = state.0.lock().unwrap();

    library::check_deletions(&conn);

    let folders: Vec<String> = {
        let mut stmt = conn.prepare("SELECT path FROM folders").unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    drop(conn);

    for folder in folders {
        let conn = state.0.lock().unwrap();
        library::scan_folder(&conn, &folder);
    }
});

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
    library::commands::add_folder,
    library::commands::remove_folder,
    library::commands::get_folders,
    library::commands::get_tracks,
    library::commands::scan_library,
    player::commands::play_track,
    player::commands::pause_track,
    player::commands::resume_track,
    player::commands::stop_track,
    player::commands::get_position,
    player::commands::get_player_state,
    player::commands::seek_to,
player::commands::set_volume,
])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}