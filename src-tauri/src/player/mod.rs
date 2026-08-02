#![allow(dead_code)]
pub mod engine;
pub mod commands;

use std::sync::{Arc, Mutex};
use serde::Serialize;
use engine::AudioEngine;

#[derive(Serialize, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct PlayerState {
    pub current_track: Option<String>,
    pub playback_state: PlaybackState,
    pub position_secs: f64,
    pub volume: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_track: None,
            playback_state: PlaybackState::Stopped,
            position_secs: 0.0,
            volume: 1.0,
        }
    }
}

pub struct Player {
    pub state: Arc<Mutex<PlayerState>>,
    pub engine: AudioEngine,
}

impl Player {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PlayerState::default())),
            engine: AudioEngine::new(),
        }
    }
}