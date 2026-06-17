export type WorkspaceTab =
  | {
      id: 'library';
      kind: 'library';
      title: 'Library';
      closable: false;
    }
  | {
      id: string;
      kind: 'reader';
      title: string;
      bookId: string;
      closable: true;
    };

export type IconComponent = typeof import('@lucide/svelte').Icon;

export type AppAction =
  | 'Import PDF'
  | 'Sync'
  | 'Settings'
  | 'Profile'
  | 'List view'
  | 'Shelf view'
  | 'Filter'
  | 'Sort'
  | 'Open reader'
  | 'Reader tool'
  | 'Export notes'
  | 'Copy citation';

export type BookRow = {
  id: string;
  title: string;
  creator: string;
  type: string;
  status: string;
  source: string;
  progress: string;
  pages: number;
  fileSize: string;
  filePath: string;
  description: string;
  tags: string[];
  addedAt: string;
  modifiedAt: string;
  fingerprint: string;
  currentPage: number;
  accent: string;
};

export type ReaderMark = {
  title: string;
  body: string;
};

export type NoteCard = {
  title: string;
  body: string;
};
