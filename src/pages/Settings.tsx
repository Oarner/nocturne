import { useState, useEffect } from 'react';
import { Button, Text, makeStyles, tokens } from '@fluentui/react-components';
import { FolderAddRegular, DeleteRegular } from '@fluentui/react-icons';
import { open } from '@tauri-apps/plugin-dialog';
import { getFolders, addFolder, removeFolder, scanLibrary } from '../lib/api';

const useStyles = makeStyles({
  root: {
    padding: '24px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  folderList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  folderRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '8px 12px',
    backgroundColor: tokens.colorNeutralBackground2,
    borderRadius: tokens.borderRadiusMedium,
  },
});

export default function Settings() {
  const styles = useStyles();
  const [folders, setFolders] = useState<string[]>([]);

  useEffect(() => {
    getFolders().then(setFolders);
  }, []);

 const handleAddFolder = async () => {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === 'string') {
    await addFolder(selected);
    await scanLibrary();
    const updated = await getFolders();
    setFolders(updated);
  }
};

  const handleRemoveFolder = async (path: string) => {
    await removeFolder(path);
    setFolders(folders.filter(f => f !== path));
  };

  return (
    <div className={styles.root}>
      <Text size={600} weight="semibold">Library Folders</Text>
      <Button
        icon={<FolderAddRegular />}
        appearance="primary"
        onClick={handleAddFolder}
      >
        Add Folder
      </Button>
      <div className={styles.folderList}>
        {folders.map(folder => (
          <div key={folder} className={styles.folderRow}>
            <Text>{folder}</Text>
            <Button
              icon={<DeleteRegular />}
              appearance="subtle"
              onClick={() => handleRemoveFolder(folder)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}