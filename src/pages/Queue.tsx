import { useOutletContext } from 'react-router-dom';
import { Text, makeStyles, tokens } from '@fluentui/react-components';
import { PlayRegular, DismissRegular } from '@fluentui/react-icons';
import { playTrack } from '../lib/api';
import type { CurrentTrack } from '../components/Layout';

interface LayoutContext {
  setCurrentTrack: (track: CurrentTrack) => void;
  setQueue: (tracks: CurrentTrack[]) => void;
  setQueueIndex: (index: number) => void;
  currentTrack: CurrentTrack | null;
  queue: CurrentTrack[];
  queueIndex: number;
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
    gridTemplateColumns: '32px 1fr 1fr 32px',
    alignItems: 'center',
    padding: '6px 12px',
    borderRadius: tokens.borderRadiusMedium,
    cursor: 'pointer',
    gap: '8px',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground2,
    },
  },
  rowActive: {
    backgroundColor: tokens.colorNeutralBackground3,
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
  removeBtn: {
    background: 'none',
    border: 'none',
    cursor: 'pointer',
    color: tokens.colorNeutralForeground3,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '4px',
    borderRadius: tokens.borderRadiusMedium,
    opacity: 0,
    ':hover': {
      color: tokens.colorNeutralForeground1,
      backgroundColor: tokens.colorNeutralBackground3,
    },
  },
  empty: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100%',
    color: tokens.colorNeutralForeground3,
  },
});

export default function Queue() {
  const styles = useStyles();
  const { queue, queueIndex, setQueue, setQueueIndex, setCurrentTrack } = useOutletContext<LayoutContext>();

  const handlePlay = async (index: number) => {
    const track = queue[index];
    setQueueIndex(index);
    setCurrentTrack(track);
    await playTrack(track.path);
  };

  const handleRemove = (index: number) => {
    const newQueue = queue.filter((_, i) => i !== index);
    setQueue(newQueue);
    if (index < queueIndex) {
      setQueueIndex(queueIndex - 1);
    } else if (index === queueIndex && newQueue.length > 0) {
      setQueueIndex(Math.min(queueIndex, newQueue.length - 1));
    }
  };

  return (
    <div className={styles.root}>
      <div className={styles.header}>
        <Text size={600} weight="semibold">Queue</Text>
        <Text size={200} style={{ display: 'block', color: 'var(--colorNeutralForeground3)' }}>
          {queue.length} tracks
        </Text>
      </div>
      {queue.length === 0 ? (
        <div className={styles.empty}>
          <Text size={300}>No tracks in queue — play something from your library</Text>
        </div>
      ) : (
        <div className={styles.list}>
          {queue.map((track, i) => (
            <div
              key={`${track.path}-${i}`}
              className={`${styles.row} ${i === queueIndex ? styles.rowActive : ''}`}
            >
              <div className={styles.numCell}>
                <Text className={styles.trackNum} size={200}>{i + 1}</Text>
                <button
                  className={styles.playBtn}
                  onClick={() => handlePlay(i)}
                >
                  <PlayRegular fontSize={14} />
                </button>
              </div>
              <Text className={styles.title} size={300}>
                {track.title ?? track.path.split('\\').pop()}
              </Text>
              <Text className={styles.secondary} size={200}>
                {track.artist ?? 'Unknown Artist'}
              </Text>
              <button
                className={styles.removeBtn}
                onClick={() => handleRemove(i)}
              >
                <DismissRegular fontSize={14} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}