import {
  Bookmark,
  Highlighter,
  NotebookTabs,
  PencilLine,
  Tag
} from '@lucide/svelte';
import type { BookRow, NoteCard, ReaderMark } from './types';

export const bookRows: BookRow[] = [
  {
    id: 'ddia',
    title: 'Designing Data-Intensive Applications',
    creator: 'Martin Kleppmann',
    type: 'Book',
    status: 'Reading',
    source: 'Books',
    progress: '68%',
    pages: 616,
    fileSize: '14.8 MB',
    filePath: '~/Library/LibrikVault/Books/designing-data-intensive-applications.pdf',
    description: 'Reference text for distributed systems, replication, partitioning, transactions, batch processing, and stream processing.',
    tags: ['distributed-systems', 'databases', 'architecture'],
    addedAt: 'Jan 12, 2026',
    modifiedAt: 'Jun 8, 2026',
    fingerprint: 'sha256:8f4d...a92c',
    currentPage: 128,
    accent: '#2663eb'
  },
  {
    id: 'pragmatic',
    title: 'The Pragmatic Programmer',
    creator: 'Hunt, Thomas',
    type: 'Book',
    status: 'Queued',
    source: 'Books',
    progress: '12%',
    pages: 352,
    fileSize: '9.2 MB',
    filePath: '~/Library/LibrikVault/Books/the-pragmatic-programmer.pdf',
    description: 'Software craft notes, engineering habits, debugging practices, and durable programming principles.',
    tags: ['software-craft', 'career', 'practice'],
    addedAt: 'Feb 4, 2026',
    modifiedAt: 'May 25, 2026',
    fingerprint: 'sha256:7bb1...e184',
    currentPage: 42,
    accent: '#0f8a5f'
  },
  {
    id: 'systems',
    title: 'Systems Performance',
    creator: 'Brendan Gregg',
    type: 'Manual',
    status: 'Indexed',
    source: 'Manuals',
    progress: '100%',
    pages: 928,
    fileSize: '38.4 MB',
    filePath: '~/Library/LibrikVault/Manuals/systems-performance.pdf',
    description: 'Operating system and application performance reference with tracing, CPU, memory, disk, and network analysis.',
    tags: ['performance', 'systems', 'reference'],
    addedAt: 'Mar 18, 2026',
    modifiedAt: 'Jun 2, 2026',
    fingerprint: 'sha256:11ac...09df',
    currentPage: 928,
    accent: '#7c3aed'
  },
  {
    id: 'attention',
    title: 'Attention Is All You Need',
    creator: 'Vaswani et al.',
    type: 'Paper',
    status: 'Annotated',
    source: 'Research',
    progress: '100%',
    pages: 15,
    fileSize: '1.1 MB',
    filePath: '~/Library/LibrikVault/Research/attention-is-all-you-need.pdf',
    description: 'Transformer architecture paper covering attention mechanisms, encoder-decoder structure, and model training details.',
    tags: ['ml', 'transformers', 'paper'],
    addedAt: 'Apr 9, 2026',
    modifiedAt: 'Jun 1, 2026',
    fingerprint: 'sha256:dc29...78bd',
    currentPage: 15,
    accent: '#c2410c'
  },
  {
    id: 'compilers',
    title: 'Engineering a Compiler',
    creator: 'Cooper, Torczon',
    type: 'Textbook',
    status: 'Missing file',
    source: 'Textbooks',
    progress: '41%',
    pages: 824,
    fileSize: '22.5 MB',
    filePath: '~/Library/LibrikVault/Textbooks/engineering-a-compiler.pdf',
    description: 'Compiler construction text covering scanning, parsing, semantic analysis, optimization, and code generation.',
    tags: ['compilers', 'textbook', 'cs'],
    addedAt: 'Apr 20, 2026',
    modifiedAt: 'May 14, 2026',
    fingerprint: 'sha256:31c0...ae44',
    currentPage: 338,
    accent: '#b42318'
  },
  {
    id: 'notes',
    title: 'Distributed Systems Reading Pack',
    creator: 'Course Pack',
    type: 'PDF Set',
    status: 'Syncing',
    source: 'Course Packs',
    progress: '83%',
    pages: 244,
    fileSize: '16.0 MB',
    filePath: '~/Library/LibrikVault/Course Packs/distributed-systems-reading-pack.pdf',
    description: 'Collected course readings on replication, consensus, storage engines, message queues, and fault tolerance.',
    tags: ['course', 'distributed-systems', 'reading-pack'],
    addedAt: 'May 2, 2026',
    modifiedAt: 'Jun 10, 2026',
    fingerprint: 'sha256:c642...0d21',
    currentPage: 202,
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
