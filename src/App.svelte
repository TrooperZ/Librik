<script lang="ts">
  import LibraryView from './components/LibraryView.svelte';
  import ReaderView from './components/ReaderView.svelte';
  import SourceSidebar from './components/SourceSidebar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import TopBar from './components/TopBar.svelte';
  import { bookRows, readerMarks, sourceSections } from './lib/mockData';
  import type { AppAction, ViewMode } from './lib/types';

  let activeView: ViewMode = 'library';
  let selectedSource = 'All PDFs';
  let selectedBookId = bookRows[0].id;

  $: selectedBook = bookRows.find((book) => book.id === selectedBookId) ?? bookRows[0];
  $: selectedSourceCount =
    sourceSections.flatMap((section) => section.items).find((item) => item.label === selectedSource)?.count ?? String(bookRows.length);

  function handleAction(_action: AppAction, _detail?: string) {
    // UI-only command surface for this shell build.
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
        books={bookRows}
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

  <StatusBar source={selectedSource} count={selectedSourceCount} selectedTitle={selectedBook.title} />
</main>
