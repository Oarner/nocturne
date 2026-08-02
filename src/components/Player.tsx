import { useState, useEffect, useCallback } from 'react';
import { Text, makeStyles, tokens, Slider } from '@fluentui/react-components';
import {
  PlayRegular,
  PauseRegular,
  NextRegular,
  PreviousRegular,
  ArrowRepeatAllRegular,
  ArrowClockwiseRegular,
  ArrowShuffleRegular,
  Speaker2Regular,
  SpeakerMuteRegular,
} from '@fluentui/react-icons';
import {
  playTrack,
  pauseTrack,
  resumeTrack,
  getPlayerState,
  seekTo,
  setVolume,
  type PlayerState,
} from '../lib/api';

const useStyles = makeStyles({
  root: {
    height: '80px',
    backgroundColor: tokens.colorNeutralBackground2,
    borderTop: `1px solid ${tokens.colorNeutralStroke2}`,
    display: 'grid',
    gridTemplateColumns: '1fr auto 1fr',
    alignItems: 'center',
    padding: '0 20px',
    gap: '16px',
    flexShrink: 0,
    minWidth: 0,
  },
  trackInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    minWidth: 0,
  },
  controls: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '4px',
  },
  buttons: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  seekRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    width: '400px',
  },
  right: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'flex-end',
    gap: '8px',
    minWidth: 0,
    overflow: 'hidden'
  },
  controlBtn: {
    background: 'none',
    border: 'none',
    cursor: 'pointer',
    color: tokens.colorNeutralForeground2,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '6px',
    borderRadius: tokens.borderRadiusMedium,
    fontSize: '18px',
    ':hover': {
      color: tokens.colorNeutralForeground1,
      backgroundColor: tokens.colorNeutralBackground3,
    },
  },
  controlBtnActive: {
    color: tokens.colorBrandForeground1,
  },
  playBtn: {
    background: 'none',
    border: 'none',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '8px',
    borderRadius: '50%',
    fontSize: '22px',
    backgroundColor: tokens.colorNeutralForeground1,
    color: tokens.colorNeutralBackground1,  // ← duplicate, remove the first one
    ':hover': {
      backgroundColor: tokens.colorNeutralForeground2,
    },
  },
  timeText: {
    color: tokens.colorNeutralForeground3,
    whiteSpace: 'nowrap',
    minWidth: '36px',
    textAlign: 'center',
  },
  volumeRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    width: '100px',
    maxWidth: '100px',
  },
});

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

type RepeatMode = 'off' | 'all' | 'one';

interface PlayerBarProps {
  currentTrack: { path: string; title: string | null; artist: string | null; duration?: number | null } | null;
  onPrevious: () => void;
  onNext: () => void;
  shuffle: boolean;
  onShuffleToggle: () => void;
  repeatMode: RepeatMode;
  onRepeatToggle: () => void;
}

export default function PlayerBar({
  currentTrack,
  onPrevious,
  onNext,
  shuffle,
  onShuffleToggle,
  repeatMode,
  onRepeatToggle,
}: PlayerBarProps) {
  const styles = useStyles();
  const [playerState, setPlayerState] = useState<PlayerState | null>(null);
  const [isSeeking, setIsSeeking] = useState(false);
  const [seekValue, setSeekValue] = useState(0);
  const [volume, setVolumeState] = useState(1);
  const [muted, setMuted] = useState(false);
  const [prevVolume, setPrevVolume] = useState(1);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const state = await getPlayerState();
        setPlayerState(state);
        if (!isSeeking) {
          setSeekValue(state.position_secs);
        }
      } catch {
        // ignore
      }
    }, 500);
    return () => clearInterval(interval);
  }, [isSeeking]);

  const handlePlayPause = useCallback(async () => {
    if (!playerState) return;
    if (playerState.is_playing) {
      await pauseTrack();
    } else if (currentTrack) {
      if (playerState.current_track === currentTrack.path) {
        await resumeTrack();
      } else {
        await playTrack(currentTrack.path);
      }
    }
  }, [playerState, currentTrack]);

  const handleSeekChange = (value: number) => {
    setIsSeeking(true);
    setSeekValue(value);
  };

  const handleSeekCommit = async (value: number) => {
    await seekTo(value);
    setIsSeeking(false);
  };

  const handleVolumeChange = async (value: number) => {
    setVolumeState(value);
    setMuted(value === 0);
    await setVolume(value);
  };

  const handleMuteToggle = async () => {
    if (muted) {
      setMuted(false);
      setVolumeState(prevVolume);
      await setVolume(prevVolume);
    } else {
      setPrevVolume(volume);
      setMuted(true);
      setVolumeState(0);
      await setVolume(0);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';
      if (isInput) return;

      if (e.code === 'Space') {
        e.preventDefault();
        handlePlayPause();
      } else if (e.code === 'ArrowLeft') {
        e.preventDefault();
        const newPos = Math.max(0, seekValue - 5);
        seekTo(newPos);
        setSeekValue(newPos);
      } else if (e.code === 'ArrowRight') {
        e.preventDefault();
        const duration = currentTrack?.duration ?? 0;
        const newPos = Math.min(duration, seekValue + 5);
        seekTo(newPos);
        setSeekValue(newPos);
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handlePlayPause, seekValue, currentTrack]);

  const isPlaying = playerState?.is_playing ?? false;
  const duration = currentTrack?.duration ?? 0;
  const remaining = Math.max(0, duration - seekValue);

  const RepeatIcon = repeatMode === 'one' ? ArrowClockwiseRegular : ArrowRepeatAllRegular;

  return (
    <div className={styles.root}>
      <div className={styles.trackInfo}>
        {currentTrack ? (
          <>
            <Text size={300} weight="semibold" truncate block>
              {currentTrack.title ?? currentTrack.path.split('\\').pop()}
            </Text>
            <Text size={200} style={{ color: 'var(--colorNeutralForeground3)' }} truncate block>
              {currentTrack.artist ?? 'Unknown Artist'}
            </Text>
          </>
        ) : (
          <Text size={200} style={{ color: 'var(--colorNeutralForeground3)' }}>
            No track selected
          </Text>
        )}
      </div>

      <div className={styles.controls}>
        <div className={styles.buttons}>
          <button
            className={`${styles.controlBtn} ${shuffle ? styles.controlBtnActive : ''}`}
            onClick={onShuffleToggle}
            title="Shuffle"
          >
            <ArrowShuffleRegular />
          </button>
          <button className={styles.controlBtn} onClick={onPrevious} title="Previous">
            <PreviousRegular />
          </button>
          <button className={styles.playBtn} onClick={handlePlayPause}>
            {isPlaying ? <PauseRegular /> : <PlayRegular />}
          </button>
          <button className={styles.controlBtn} onClick={onNext} title="Next">
            <NextRegular />
          </button>
          <button
            className={`${styles.controlBtn} ${repeatMode !== 'off' ? styles.controlBtnActive : ''}`}
            onClick={onRepeatToggle}
            title="Repeat"
          >
            <RepeatIcon />
          </button>
        </div>

        <div className={styles.seekRow}>
          <Text size={100} className={styles.timeText}>{formatTime(seekValue)}</Text>
          <Slider
            style={{ flex: 1 }}
            min={0}
            max={duration || 1}
            value={seekValue}
            step={0.1}
            onChange={(_, data) => handleSeekChange(data.value)}
            onPointerUp={() => handleSeekCommit(seekValue)}
          />
          <Text size={100} className={styles.timeText}>-{formatTime(remaining)}</Text>
        </div>
      </div>

      <div className={styles.right}>
        <div className={styles.volumeRow}>
          <button className={styles.controlBtn} onClick={handleMuteToggle}>
            {muted || volume === 0 ? <SpeakerMuteRegular /> : <Speaker2Regular />}
          </button>
          <Slider
            style={{ flex: 1 }}
            min={0}
            max={1}
            step={0.01}
            value={muted ? 0 : volume}
            onChange={(_, data) => handleVolumeChange(data.value)}
          />
        </div>
      </div>
    </div>
  );
}