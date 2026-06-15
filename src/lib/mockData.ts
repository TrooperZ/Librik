import {
  Archive,
  BookOpen,
  Bookmark,
  Boxes,
  CircleAlert,
  Clock3,
  Cloud,
  FileText,
  FolderGit2,
  GraduationCap,
  Highlighter,
  Library,
  NotebookTabs,
  PencilLine,
  ScrollText,
  Share2,
  Tag
} from '@lucide/svelte';
import type { BookRow, NoteCard, ReaderMark, SourceSection } from './types';

export const sourceSections: SourceSection[] = [
  {
    label: 'Sources',
    items: [
      { label: 'All PDFs', icon: Library, count: '842', selected: true },
      { label: 'Local Library', icon: Archive, count: '510' },
      { label: 'GitHub Repos', icon: FolderGit2, count: '94' },
      { label: 'Google Drive', icon: Cloud, count: '218' },
      { label: 'Shared Collections', icon: Share2, count: '20' }
    ]
  },
  {
    label: 'Collections',
    items: [
      { label: 'Books', icon: BookOpen, count: '344' },
      { label: 'Textbooks', icon: GraduationCap, count: '88' },
      { label: 'Research Articles', icon: FileText, count: '273' },
      { label: 'Course Packs', icon: Boxes, count: '31' },
      { label: 'Manuals', icon: ScrollText, count: '106' },
      { label: 'Reading Queue', icon: Bookmark, count: '17' }
    ]
  },
  {
    label: 'Smart',
    items: [
      { label: 'Recently Opened', icon: Clock3, count: '24' },
      { label: 'Bookmarked', icon: Bookmark, count: '66' },
      { label: 'Annotated', icon: Highlighter, count: '128' },
      { label: 'Missing Files', icon: CircleAlert, count: '9' }
    ]
  }
];

export const bookRows: BookRow[] = [
  {
    id: 'ddia',
    title: 'Designing Data-Intensive Applications',
    creator: 'Martin Kleppmann',
    type: 'Book',
    status: 'Reading',
    source: 'Local',
    progress: '68%',
    accent: '#2663eb'
  },
  {
    id: 'pragmatic',
    title: 'The Pragmatic Programmer',
    creator: 'Hunt, Thomas',
    type: 'Book',
    status: 'Queued',
    source: 'Drive',
    progress: '12%',
    accent: '#0f8a5f'
  },
  {
    id: 'systems',
    title: 'Systems Performance',
    creator: 'Brendan Gregg',
    type: 'Manual',
    status: 'Indexed',
    source: 'Local',
    progress: '100%',
    accent: '#7c3aed'
  },
  {
    id: 'attention',
    title: 'Attention Is All You Need',
    creator: 'Vaswani et al.',
    type: 'Paper',
    status: 'Annotated',
    source: 'GitHub',
    progress: '100%',
    accent: '#c2410c'
  },
  {
    id: 'compilers',
    title: 'Engineering a Compiler',
    creator: 'Cooper, Torczon',
    type: 'Textbook',
    status: 'Missing file',
    source: 'Shared',
    progress: '41%',
    accent: '#b42318'
  },
  {
    id: 'notes',
    title: 'Distributed Systems Reading Pack',
    creator: 'Course Pack',
    type: 'PDF Set',
    status: 'Syncing',
    source: 'Drive',
    progress: '83%',
    accent: '#475467'
  }
];

export const readerMarks: ReaderMark[] = [
  { title: 'Chapter 5 Replication', body: '3 highlights, 1 margin note' },
  { title: 'Chapter 6 Partitioning', body: '2 highlights, bookmark' },
  { title: 'Chapter 7 Transactions', body: '5 highlights, quote staged' }
];

export const noteCards: NoteCard[] = [
  {
    title: 'Active annotation',
    body: '“Replication is the act of keeping a copy of the same data on multiple machines...”'
  },
  {
    title: 'Notebook links',
    body: 'Distributed Systems, Databases, Architecture Decisions'
  },
  {
    title: 'Extracted quotes',
    body: 'Use this as a staging area before exporting to Markdown, citations, or study notes.'
  },
  {
    title: 'Reader tools',
    body: 'Highlight, underline, free text note, bookmark, area selection, export notes.'
  }
];

export const readerTools = [
  { label: 'Highlight', icon: Highlighter },
  { label: 'Note', icon: PencilLine },
  { label: 'Bookmark', icon: Bookmark },
  { label: 'Notebook', icon: NotebookTabs },
  { label: 'Tags', icon: Tag }
];
