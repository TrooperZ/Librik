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

export type BookRow = {
  id: string;
  title: string;
  creator: string;
  type: string;
  status: string;
  source: string;
  folderId: string | null;
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

export type FolderRow = {
  id: string;
  name: string;
  parentId: string | null;
  color: string | null;
  sortOrder: number;
  documentCount: number;
};
