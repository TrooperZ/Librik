<script lang="ts">
    import {
        Download,
        Filter,
        Grid2X2,
        List,
        MoreHorizontal,
        Plus,
    } from "@lucide/svelte";
    import type { AppAction, BookRow } from "../lib/types";

    export let books: BookRow[];
    export let selectedBookId: string;
    export let onSelectBook: (bookId: string) => void;
    export let onOpenReader: (bookId: string) => void;
    export let onAction: (action: AppAction, detail?: string) => void;
</script>

<section class="library-view" aria-label="Library management view">
    <header class="view-heading">
        <div>
            <h1>All PDFs</h1>
            <p>{books.length} PDFs</p>
        </div>

        <div class="view-actions" aria-label="Library view controls">
            <button
                type="button"
                title="Import PDF"
                onclick={() => onAction("Import PDF")}
            >
                <Download size={16} />
            </button>
            <button
                type="button"
                class="text-command"
                title="Add Folder"
                onclick={() => onAction("Filter", "Add Folder")}
            >
                <Plus size={15} />
                Folder
            </button>
            <button
                type="button"
                class="selected"
                title="List view"
                onclick={() => onAction("List view")}
            >
                <List size={16} />
            </button>
            <button
                type="button"
                title="Shelf view"
                onclick={() => onAction("Shelf view")}
            >
                <Grid2X2 size={16} />
            </button>
            <button
                type="button"
                title="Filter"
                onclick={() => onAction("Filter")}
            >
                <Filter size={16} />
            </button>
            <button
                type="button"
                class="sort-button"
                onclick={() => onAction("Sort")}>Sort: Recent</button
            >
        </div>
    </header>

    <div class="library-table" role="table" aria-label="Books and PDFs">
        <div class="table-header" role="row">
            <span>Cover</span>
            <span>Title</span>
            <span>Tags</span>
            <span>Progress</span>
            <span>Folder</span>
            <span>Modified</span>
            <span></span>
        </div>

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
                <span class="cover-chip" style={`--accent:${book.accent}`}
                ></span>
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
                <span>{book.source}</span>
                <span>{book.modifiedAt}</span>
                <span class="row-actions">
                    <button
                        type="button"
                        title="Open reader"
                        onclick={(event) => {
                            event.stopPropagation();
                            onOpenReader(book.id);
                        }}
                    >
                        Read
                    </button>
                    <button
                        type="button"
                        title="More actions"
                        aria-label={`More actions for ${book.title}`}
                        onclick={(event) => {
                            event.stopPropagation();
                            onAction("Open reader", book.title);
                        }}
                    >
                        <MoreHorizontal size={15} />
                    </button>
                </span>
            </div>
        {/each}
    </div>
</section>
