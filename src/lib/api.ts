import { invoke } from '@tauri-apps/api/core';

export interface Track {
  id: number;
  path: string;
  title: string | null;
  artist: string | null;
  album: string | null;
  album_artist: string | null;
  duration_secs: number | null;
  track_number: number | null;
  disc_number: number | null;
  year: number | null;
  genre: string | null;
  has_cover: boolean;
}

export const getFolders = (): Promise<string[]> =>
  invoke('get_folders');

export const addFolder = (path: string): Promise<void> =>
  invoke('add_folder', { path });

export const removeFolder = (path: string): Promise<void> =>
  invoke('remove_folder', { path });

export const getTracks = (): Promise<Track[]> =>
  invoke('get_tracks');
export const scanLibrary = (): Promise<void> =>
  invoke('scan_library');
export interface PlayerState {
  current_track: string | null;
  is_playing: boolean;
  position_secs: number;
  volume: number;
}

export const playTrack = (path: string): Promise<void> =>
  invoke('play_track', { path });

export const pauseTrack = (): Promise<void> =>
  invoke('pause_track');

export const resumeTrack = (): Promise<void> =>
  invoke('resume_track');

export const stopTrack = (): Promise<void> =>
  invoke('stop_track');

export const getPosition = (): Promise<number> =>
  invoke('get_position');

export const getPlayerState = (): Promise<PlayerState> =>
  invoke('get_player_state');
export const seekTo = (position: number): Promise<void> =>
  invoke('seek_to', { position });

export const setVolume = (volume: number): Promise<void> =>
  invoke('set_volume', { volume });