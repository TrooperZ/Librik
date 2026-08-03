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
