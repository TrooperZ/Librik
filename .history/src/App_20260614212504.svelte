<script lang="ts">
  import LibraryView from './components/LibraryView.svelte';
  import PdfDetailsPanel from './components/PdfDetailsPanel.svelte';
  import ReaderView from './components/ReaderView.svelte';
  import SourceSidebar from './components/SourceSidebar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import TopBar from './components/TopBar.svelte';
  import { bookRows, noteCards, readerMarks, sourceSections } from './lib/mockData';
  import type { AppAction, BookRow, ViewMode } from './lib/types';

  let activeView: ViewMode = 'library';
  let selectedSource = 'All PDFs';
  let selectedBookId = bookRows[0].id;
  let books: BookRow[] = bookRows;
  let statusMessage = 'Ready';

  $: selectedBook = books.find((book) => book.id === selectedBookId) ?? books[0];
  $: selectedSourceCount =
    sourceSections.flatMap((section) => section.items).find((item) => item.label === selectedSource)?.count ?? String(books.length);

  function createMockBook(): BookRow {
    const number = books.filter((book) => book.id.startsWith('mock-import')).length + 1;
    return {
      id: `mock-import-${number}`,
      title: `Imported PDF ${number}`,
      creator: 'Unknown Author',
      type: 'PDF',
      status: 'Queued',
      source: 'Local',
      progress: '0%',
      pages: 186,
      fileSize: '6.4 MB',
      filePath: `~/Library/Librik/Inbox/imported-pdf-${number}.pdf`,
      description: 'Newly added PDF awaiting review, tagging, and reading progress.',
      tags: ['inbox'],
      addedAt: 'Today',
      modifiedAt: 'Today',
      fingerprint: `sha256:mock...${String(number).padStart(4, '0')}`,
      currentPage: 1,
      accent: '#2563eb'
    };
  }

  function handleAction(action: AppAction, detail?: string) {
    if (action === 'Import PDF') {
      const book = createMockBook();
      books = [book, ...books];
      selectedBookId = book.id;
      activeView = 'library';
      statusMessage = `${book.title} added to inbox`;
      return;
    }

    statusMessage = detail ? `${action}: ${detail}` : action;
  }

  function handleSourceSelect(source: string) {
    selectedSource = source;
  }

  function handleBookSelect(bookId: string) {
    selectedBookId = bookId;
  }

  function handleOpenReader(bookId: string) {
    selectedBookId = bookId;
    activeView = 'reader';
    handleAction('Open reader');
  }

  function updateBook(bookId: string, update: (book: BookRow) => BookRow) {
    books = books.map((book) => (book.id === bookId ? update(book) : book));
  }

  function handleAddTag(bookId: string, tag: string) {
    updateBook(bookId, (book) => {
      if (book.tags.includes(tag)) return book;
      return { ...book, tags: [...book.tags, tag] };
    });
    statusMessage = `Tag added: ${tag}`;
  }

  function handleRemoveTag(bookId: string, tag: string) {
    updateBook(bookId, (book) => ({ ...book, tags: book.tags.filter((value) => value !== tag) }));
    statusMessage = `Tag removed: ${tag}`;
  }

  function handleSetProgress(bookId: string, progress: string) {
    updateBook(bookId, (book) => ({
      ...book,
      progress,
      currentPage: Math.max(1, Math.round((Number.parseInt(progress) / 100) * book.pages)),
      status: progress === '100%' ? 'Annotated' : 'Reading'
    }));
    statusMessage = `Progress set to ${progress}`;
  }

  function handleSetStatus(bookId: string, status: string) {
    updateBook(bookId, (book) => ({ ...book, status }));
    statusMessage = `Status set to ${status}`;
  }
</script>

<main class="app-shell" aria-label="Librik">
  <TopBar onAction={handleAction} />

  <SourceSidebar sections={sourceSections} {selectedSource} onSelect={handleSourceSelect} />

  <section class="workspace" aria-label="Primary workspace">
    <nav class="workspace-tabs" aria-label="Workspace switcher">
      <button
        type="button"
        class:active={activeView === 'library'}
        aria-pressed={activeView === 'library'}
        onclick={() => {
          activeView = 'library';
          handleAction('List view');
        }}
      >
        Library
      </button>
      <button
        type="button"
        class:active={activeView === 'reader'}
        aria-pressed={activeView === 'reader'}
        onclick={() => {
          activeView = 'reader';
          handleAction('Open reader');
        }}
      >
        Reader
      </button>
    </nav>

    {#if activeView === 'library'}
      <LibraryView
        {books}
        {selectedBookId}
        onSelectBook={handleBookSelect}
        onOpenReader={handleOpenReader}
        onAction={handleAction}
      />
    {:else}
      <ReaderView
        book={selectedBook}
        marks={readerMarks}
        onAction={handleAction}
      />
    {/if}
  </section>

  <PdfDetailsPanel
    book={selectedBook}
    notes={noteCards}
    onAction={handleAction}
    onOpenReader={handleOpenReader}
    onAddTag={handleAddTag}
    onRemoveTag={handleRemoveTag}
    onSetProgress={handleSetProgress}
    onSetStatus={handleSetStatus}
  />

  <StatusBar source={selectedSource} count={selectedSourceCount} selectedTitle={selectedBook.title} message={statusMessage} />
</main>
