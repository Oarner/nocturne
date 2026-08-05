import { useState } from 'react';
import { makeStyles, tokens } from '@fluentui/react-components';
import { MusicNote2Regular, ListRegular, SettingsRegular } from '@fluentui/react-icons';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import PlayerBar from './Player';

const useStyles = makeStyles({
  root: {
    display: 'flex',
    height: '100vh',
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: 'hidden',
  },
  sidebar: {
    width: '48px',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    paddingTop: '12px',
    gap: '4px',
    backgroundColor: tokens.colorNeutralBackground2,
    flexShrink: 0,
  },
  navItem: {
    width: '36px',
    height: '36px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: tokens.borderRadiusMedium,
    cursor: 'pointer',
    color: tokens.colorNeutralForeground2,
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground3,
      color: tokens.colorNeutralForeground1,
    },
  },
  navItemActive: {
    backgroundColor: tokens.colorNeutralBackground3,
    color: tokens.colorNeutralForeground1,
  },
  main: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
  },
  content: {
    flex: 1,
    overflow: 'auto',
  },
});

const navItems = [
  { path: '/', icon: <MusicNote2Regular />, label: 'Library' },
  { path: '/queue', icon: <ListRegular />, label: 'Queue' },
  { path: '/settings', icon: <SettingsRegular />, label: 'Settings' },
];

export interface CurrentTrack {
  path: string;
  title: string | null;
  artist: string | null;
  duration?: number | null;
}

type RepeatMode = 'off' | 'all' | 'one';

export default function Layout() {
  const styles = useStyles();
  const navigate = useNavigate();
  const location = useLocation();
  const [currentTrack, setCurrentTrack] = useState<CurrentTrack | null>(null);
  const [queue, setQueue] = useState<CurrentTrack[]>([]);
  const [queueIndex, setQueueIndex] = useState(0);
  const [shuffle, setShuffle] = useState(false);
  const [repeatMode, setRepeatMode] = useState<RepeatMode>('off');

  const handleNext = () => {
    if (queue.length === 0) return;
    const nextIndex = (queueIndex + 1) % queue.length;
    setQueueIndex(nextIndex);
    setCurrentTrack(queue[nextIndex]);
  };

  const handlePrevious = () => {
    if (queue.length === 0) return;
    const prevIndex = (queueIndex - 1 + queue.length) % queue.length;
    setQueueIndex(prevIndex);
    setCurrentTrack(queue[prevIndex]);
  };

  const handleRepeatToggle = () => {
    setRepeatMode(prev =>
      prev === 'off' ? 'all' : prev === 'all' ? 'one' : 'off'
    );
  };

  return (
    <div className={styles.root}>
      <div className={styles.sidebar}>
        {navItems.map(item => (
          <div
            key={item.path}
            className={`${styles.navItem} ${location.pathname === item.path ? styles.navItemActive : ''}`}
            onClick={() => navigate(item.path)}
            title={item.label}
          >
            {item.icon}
          </div>
        ))}
      </div>
      <div className={styles.main}>
        <div className={styles.content}>
          <Outlet context={{ setCurrentTrack, setQueue, setQueueIndex, currentTrack, queue, queueIndex }} />
        </div>
        <PlayerBar
          currentTrack={currentTrack}
          onPrevious={handlePrevious}
          onNext={handleNext}
          shuffle={shuffle}
          onShuffleToggle={() => setShuffle(s => !s)}
          repeatMode={repeatMode}
          onRepeatToggle={handleRepeatToggle}
        />
      </div>
    </div>
  );
}