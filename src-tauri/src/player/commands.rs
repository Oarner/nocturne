use tauri::State;
use crate::player::{Player, PlaybackState};

#[tauri::command]
pub fn play_track(path: String, state: State<Player>) -> Result<(), String> {
    state.engine.stop();
    state.engine.play(&path)?;
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.current_track = Some(path);
    player_state.playback_state = PlaybackState::Playing;
    player_state.position_secs = 0.0;
    Ok(())
}

#[tauri::command]
pub fn pause_track(state: State<Player>) -> Result<(), String> {
    state.engine.pause();
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.playback_state = PlaybackState::Paused;
    Ok(())
}

#[tauri::command]
pub fn resume_track(state: State<Player>) -> Result<(), String> {
    state.engine.resume();
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.playback_state = PlaybackState::Playing;
    Ok(())
}

#[tauri::command]
pub fn stop_track(state: State<Player>) -> Result<(), String> {
    state.engine.stop();
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.current_track = None;
    player_state.playback_state = PlaybackState::Stopped;
    player_state.position_secs = 0.0;
    Ok(())
}

#[tauri::command]
pub fn get_position(state: State<Player>) -> f64 {
    state.engine.position()
}

#[tauri::command]
pub fn get_player_state(state: State<Player>) -> Result<PlayerStateDto, String> {
    let player_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(PlayerStateDto {
        current_track: player_state.current_track.clone(),
        is_playing: player_state.playback_state == PlaybackState::Playing,
        position_secs: state.engine.position(),
        volume: player_state.volume,
    })
}

#[derive(serde::Serialize)]
pub struct PlayerStateDto {
    pub current_track: Option<String>,
    pub is_playing: bool,
    pub position_secs: f64,
    pub volume: f32,
}
#[tauri::command]
pub fn seek_to(position: f64, state: State<Player>) -> Result<(), String> {
    state.engine.seek(position);
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.position_secs = position;
    Ok(())
}

#[tauri::command]
pub fn set_volume(volume: f32, state: State<Player>) -> Result<(), String> {
    state.engine.set_volume(volume);
    let mut player_state = state.state.lock().map_err(|e| e.to_string())?;
    player_state.volume = volume;
    Ok(())
}