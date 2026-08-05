import { useEffect, useState } from 'react';
import { Text, makeStyles, tokens, Spinner } from '@fluentui/react-components';
import { PlayRegular } from '@fluentui/react-icons';
import { useOutletContext } from 'react-router-dom';
import { getTracks, playTrack, type Track } from '../lib/api';
import type { CurrentTrack } from '../components/Layout';

interface LayoutContext {
  setCurrentTrack: (track: CurrentTrack) => void;
  setQueue: (tracks: CurrentTrack[]) => void;
  setQueueIndex: (index: number) => void;
  currentTrack: CurrentTrack | null;
}

const useStyles = makeStyles({
  root: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    overflow: 'hidden',
  },
  header: {
    padding: '16px 20px 8px',
    flexShrink: 0,
  },
  list: {
    flex: 1,
    overflowY: 'auto',
    padding: '0 8px',
  },
  row: {
    display: 'grid',
    gridTemplateColumns: '32px 1fr 1fr 1fr 60px',
    alignItems: 'center',
    padding: '6px 12px',
    borderRadius: tokens.borderRadiusMedium,
    cursor: 'pointer',
    gap: '8px',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground2,
    },
    ':hover .trackNum': {
      opacity: 0,
    },
    ':hover .playBtn': {
      opacity: 1,
    },
  },
  numCell: {
    position: 'relative',
    width: '32px',
    height: '20px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'flex-end',
  },
  trackNum: {
    color: tokens.colorNeutralForeground3,
    transition: 'opacity 0.1s',
  },
  playBtn: {
    position: 'absolute',
    right: 0,
    opacity: 0,
    background: 'none',
    border: 'none',
    cursor: 'pointer',
    color: tokens.colorNeutralForeground1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 0,
    transition: 'opacity 0.1s',
  },
  title: {
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  secondary: {
    color: tokens.colorNeutralForeground3,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  duration: {
    color: tokens.colorNeutralForeground3,
    textAlign: 'right',
  },
  center: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100%',
  },
});

function formatDuration(secs: number | null): string {
  if (!secs) return '--:--';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export default function Library() {
  const styles = useStyles();
  const [tracks, setTracks] = useState<Track[]>([]);
  const [loading, setLoading] = useState(true);
  const { setCurrentTrack, setQueue, setQueueIndex } = useOutletContext<LayoutContext>();

  useEffect(() => {
    getTracks().then(t => {
      setTracks(t);
      setLoading(false);
    });
  }, []);

  const handlePlay = async (track: Track, index: number) => {
    const q = tracks.map(t => ({
      path: t.path,
      title: t.title,
      artist: t.artist,
      duration: t.duration_secs,
    }));
    setQueue(q);
    setQueueIndex(index);
   setCurrentTrack({ path: track.path, title: track.title, artist: track.artist, duration: track.duration_secs });
    await playTrack(track.path);
  };

  return (
    <div className={styles.root}>
      <div className={styles.header}>
        <Text size={600} weight="semibold">Library</Text>
        <Text size={200} style={{ display: 'block', color: 'var(--colorNeutralForeground3)' }}>
          {tracks.length} tracks
        </Text>
      </div>
      {loading ? (
        <div className={styles.center}><Spinner /></div>
      ) : (
        <div className={styles.list}>
          {tracks.map((track, i) => (
            <div key={track.id} className={styles.row}>
              <div className={styles.numCell}>
                <Text className={styles.trackNum} size={200}>{i + 1}</Text>
                <button
                  className={styles.playBtn}
                  onClick={(e) => { e.stopPropagation(); handlePlay(track, i); }}
                >
                  <PlayRegular fontSize={14} />
                </button>
              </div>
              <Text className={styles.title} size={300}>{track.title ?? track.path.split('\\').pop()}</Text>
              <Text className={styles.secondary} size={200}>{track.artist ?? 'Unknown Artist'}</Text>
              <Text className={styles.secondary} size={200}>{track.album ?? 'Unknown Album'}</Text>
              <Text className={styles.duration} size={200}>{formatDuration(track.duration_secs)}</Text>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}