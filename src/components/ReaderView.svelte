<script lang="ts">
  import { Bookmark, Highlighter, NotebookTabs, PencilLine, Tag } from '@lucide/svelte';
  import type { AppAction, BookRow, NoteCard, ReaderMark } from '../lib/types';
  import { noteCards } from '../lib/mockData';

  export let book: BookRow;
  export let marks: ReaderMark[];
  export let onAction: (action: AppAction, detail?: string) => void;

  const notes: NoteCard[] = noteCards;
  const lines = [390, 425, 408, 365, 430, 410, 380, 425, 402, 335, 430, 406, 375, 300];
  let selectedTool = 'Highlight';

  function selectTool(tool: string) {
    selectedTool = tool;
    onAction('Reader tool');
  }
</script>

<section class="reader-view" aria-label="Reader and annotation view">
  <aside class="reader-rail" aria-label="Reader quick marks">
    <h2>Outline</h2>
    <div class="chapter-list">
      <button type="button" class="active" onclick={() => onAction('Reader tool', 'Chapter 5 Replication')}>
        <span>Chapter 5 Replication</span>
        <strong>3</strong>
      </button>
      <button type="button" onclick={() => onAction('Reader tool', 'Chapter 6 Partitioning')}>
        <span>Chapter 6 Partitioning</span>
        <strong>2</strong>
      </button>
      <button type="button" onclick={() => onAction('Reader tool', 'Chapter 7 Transactions')}>
        <span>Chapter 7 Transactions</span>
        <strong>5</strong>
      </button>
    </div>

    <h2>Quick marks</h2>
    {#each marks as mark}
      <button type="button" class="mark-card" onclick={() => onAction('Reader tool', mark.title)}>
        <strong>{mark.title}</strong>
        <span>{mark.body}</span>
      </button>
    {/each}
  </aside>

  <div class="reader-canvas" aria-label="Reader canvas">
    <div class="reader-toolstrip">
      <button type="button" class:active={selectedTool === 'Highlight'} aria-pressed={selectedTool === 'Highlight'} onclick={() => selectTool('Highlight')} onpointerdown={() => selectTool('Highlight')}>
        <Highlighter size={15} />
        <span>Highlight</span>
      </button>
      <button type="button" class:active={selectedTool === 'Note'} aria-pressed={selectedTool === 'Note'} onclick={() => selectTool('Note')} onpointerdown={() => selectTool('Note')}>
        <PencilLine size={15} />
        <span>Note</span>
      </button>
      <button type="button" class:active={selectedTool === 'Bookmark'} aria-pressed={selectedTool === 'Bookmark'} onclick={() => selectTool('Bookmark')} onpointerdown={() => selectTool('Bookmark')}>
        <Bookmark size={15} />
        <span>Bookmark</span>
      </button>
      <button type="button" class:active={selectedTool === 'Notebook'} aria-pressed={selectedTool === 'Notebook'} onclick={() => selectTool('Notebook')} onpointerdown={() => selectTool('Notebook')}>
        <NotebookTabs size={15} />
        <span>Notebook</span>
      </button>
      <button type="button" class:active={selectedTool === 'Tags'} aria-pressed={selectedTool === 'Tags'} onclick={() => selectTool('Tags')} onpointerdown={() => selectTool('Tags')}>
        <Tag size={15} />
        <span>Tags</span>
      </button>
    </div>

    <article class="pdf-page" aria-label={book.title}>
      <div class="bookmark-ribbon" aria-hidden="true"></div>
      <h1>{book.title}</h1>
      {#each lines as width, index}
        <span
          class:highlight-line={index === 4}
          class="pdf-line"
          style={`--line-width:${width}px`}
        ></span>
      {/each}
      <button type="button" class="margin-note" onclick={() => onAction('Reader tool', 'Margin note')}>N1</button>
    </article>

    <div class="reader-bottom-controls">
      <button type="button" onclick={() => onAction('Reader tool', 'Previous page')}>Prev</button>
      <span>Page 128 of 614</span>
      <button type="button" onclick={() => onAction('Reader tool', 'Next page')}>Next</button>
      <span>Zoom 100%</span>
      <span>Search idle</span>
    </div>
  </div>

  <aside class="notes-panel" aria-label="Reader notes panel">
    <header>
      <h2>Notebook</h2>
      <input placeholder="Search notes" aria-label="Search notes" onclick={() => onAction('Filter')} />
    </header>

    {#each notes as note}
      <button type="button" class="note-card" onclick={() => onAction('Reader tool', note.title)}>
        <strong>{note.title}</strong>
        <span>{note.body}</span>
      </button>
    {/each}

    <div class="note-actions">
      <button type="button" onclick={() => onAction('Export notes')}>Export Notes</button>
      <button type="button" onclick={() => onAction('Copy citation')}>Copy Citation</button>
    </div>
  </aside>
</section>
