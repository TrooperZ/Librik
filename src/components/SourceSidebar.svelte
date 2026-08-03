<script lang="ts">
    import { Library, Search } from "@lucide/svelte";
    import type { BookRow } from "../lib/types";
    import type { ActivityView } from "./ActivityRail.svelte";

    export let books: BookRow[];
    export let activeActivity: ActivityView;
    export let selectedFilter: string;
    export let selectedTag: string;
    export let searchQuery: string;
    export let onFilter: (filter: string) => void;
    export let onSearch: (query: string) => void;
    export let onTag: (tag: string) => void;
    export let onOpenBook: (bookId: string) => void;

    let tagSearchQuery = "";

    $: filters = [
        { id: "all", label: "All PDFs", count: books.length },
        {
            id: "reading",
            label: "Reading",
            count: books.filter((book) => book.status === "Reading").length,
        },
        {
            id: "queued",
            label: "Queued",
            count: books.filter((book) => book.status === "Queued").length,
        },
        {
            id: "finished",
            label: "Finished",
            count: books.filter((book) => book.status === "Finished").length,
        },
        {
            id: "missing",
            label: "Missing files",
            count: books.filter((book) => book.status === "Missing file").length,
        },
    ];
    $: tagCounts = books.reduce<Record<string, number>>((counts, book) => {
        for (const tag of book.tags) counts[tag] = (counts[tag] ?? 0) + 1;
        return counts;
    }, {});
    $: tags = Object.entries(tagCounts)
        .filter(([tag]) =>
            tag.toLowerCase().includes(tagSearchQuery.trim().toLowerCase()),
        )
        .sort(([left], [right]) => left.localeCompare(right));
</script>

<aside class="source-sidebar" aria-label="Library sources">
    {#if activeActivity === "library"}
        <section class="source-section library-tree-section" aria-label="Library filters">
            <header class="library-panel-header"><span>Library</span></header>
            <div class="vault-tree">
                {#each filters as filter}
                    <button
                        type="button"
                        class="tree-row"
                        class:active={selectedFilter === filter.id}
                        aria-pressed={selectedFilter === filter.id}
                        onclick={() => onFilter(filter.id)}
                    >
                        <span class="tree-disclosure" aria-hidden="true"></span>
                        <Library size={16} strokeWidth={1.8} />
                        <span>{filter.label}</span>
                        <strong>{filter.count}</strong>
                    </button>
                {/each}
            </div>
        </section>
    {:else if activeActivity === "search"}
        <section class="activity-panel search-panel" aria-label="Search library">
            <label class="compact-search">
                <Search size={15} strokeWidth={1.8} />
                <input
                    value={searchQuery}
                    placeholder="Title, author, description, or tag"
                    oninput={(event) => onSearch(event.currentTarget.value)}
                />
            </label>
        </section>
    {:else if activeActivity === "tags"}
        <section class="tags-panel" aria-label="Tags">
            <div class="tag-search-row">
                <label class="compact-search">
                    <Search size={15} strokeWidth={1.8} />
                    <input bind:value={tagSearchQuery} placeholder="Search tags" />
                </label>
            </div>

            <div class="tag-check-list" aria-label="Tag list">
                {#if tags.length === 0}
                    <p class="empty-filter">No tags yet.</p>
                {/if}
                {#each tags as [tag, count]}
                    <button
                        type="button"
                        class="tag-check-row"
                        class:active={selectedTag === tag}
                        aria-pressed={selectedTag === tag}
                        onclick={() => onTag(selectedTag === tag ? "" : tag)}
                    >
                        <span>{tag}</span>
                        <strong>{count}</strong>
                    </button>
                {/each}
            </div>
        </section>
    {:else if activeActivity === "recent"}
        <section class="recent-panel" aria-label="Recently imported files">
            {#if books.length === 0}
                <p class="empty-filter">No imported PDFs yet.</p>
            {/if}
            {#each books.slice(0, 10) as book}
                <button type="button" class="recent-file-row" onclick={() => onOpenBook(book.id)}>
                    <span>{book.title}</span>
                    <small>{book.creator} · {book.addedAt}</small>
                </button>
            {/each}
        </section>
    {:else if activeActivity === "settings"}
        <section class="source-section activity-panel" aria-label="Workspace settings">
            <h2>Workspace</h2>
            <dl class="workspace-facts">
                <div><dt>Storage</dt><dd>Managed vault</dd></div>
                <div><dt>Documents</dt><dd>{books.length}</dd></div>
                <div><dt>Database</dt><dd>SQLite schema v2</dd></div>
            </dl>
        </section>
    {/if}
</aside>
