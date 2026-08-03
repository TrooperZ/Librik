<script lang="ts">
    import { Download } from "@lucide/svelte";
    import type { BookRow } from "../lib/types";

    export let title: string;
    export let books: BookRow[];
    export let selectedBookId: string;
    export let onSelectBook: (bookId: string) => void;
    export let onOpenReader: (bookId: string) => void;
    export let onImport: () => void;
</script>

<section class="library-view" aria-label="Library management view">
    <header class="view-heading">
        <div>
            <h1>{title}</h1>
            <p>{books.length} {books.length === 1 ? "PDF" : "PDFs"}</p>
        </div>

        <div class="view-actions" aria-label="Library actions">
            <button type="button" title="Import PDF" onclick={onImport}>
                <Download size={16} />
                Import
            </button>
        </div>
    </header>

    <div class="library-table" role="table" aria-label="PDF library">
        <div class="table-header" role="row">
            <span>Cover</span>
            <span>Title</span>
            <span>Tags</span>
            <span>Progress</span>
            <span>Folder</span>
            <span>Modified</span>
            <span></span>
        </div>

        {#if books.length === 0}
            <div class="library-empty">
                <strong>No matching PDFs</strong>
                <span>Import a PDF or clear the current filter.</span>
            </div>
        {/if}

        {#each books as book}
            <div
                class="book-row"
                class:selected={selectedBookId === book.id}
                role="button"
                tabindex="0"
                onclick={() => onSelectBook(book.id)}
                onkeydown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        onSelectBook(book.id);
                    }
                }}
                ondblclick={() => onOpenReader(book.id)}
            >
                <span class="cover-chip" style={`--accent:${book.accent}`}></span>
                <span class="title-cell">
                    <strong>{book.title}</strong>
                    <small>{book.creator} · {book.type}</small>
                </span>
                <span class="tag-cell">
                    {#each book.tags.slice(0, 2) as tag}
                        <em>{tag}</em>
                    {/each}
                </span>
                <span class="progress-cell">
                    <strong>{book.progress}</strong>
                    <span><i style={`width:${book.progress}`}></i></span>
                </span>
                <span>{book.source || "Unfiled"}</span>
                <span>{book.modifiedAt}</span>
                <span class="row-actions">
                    <button
                        type="button"
                        title={`Read ${book.title}`}
                        onclick={(event) => {
                            event.stopPropagation();
                            onOpenReader(book.id);
                        }}
                    >Read</button>
                </span>
            </div>
        {/each}
    </div>
</section>
