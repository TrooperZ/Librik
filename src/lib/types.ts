export type ViewMode = 'library' | 'reader';

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

export type SourceItem = {
  label: string;
  icon: IconComponent;
  count?: string;
  selected?: boolean;
};

export type SourceSection = {
  label: string;
  items: SourceItem[];
};

export type BookRow = {
  id: string;
  title: string;
  creator: string;
  type: string;
  status: string;
  source: string;
  progress: string;
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
