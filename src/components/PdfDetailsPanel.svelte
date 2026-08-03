<script lang="ts">
    import { BookOpen, Tag, X } from "@lucide/svelte";
    import type { BookRow } from "../lib/types";

    export let book: BookRow;
    export let onOpenReader: (bookId: string) => void;
    export let onAddTag: (bookId: string, tag: string) => void;
    export let onRemoveTag: (bookId: string, tag: string) => void;
    export let onSetProgress: (bookId: string, progress: string) => void;
    export let onSetStatus: (bookId: string, status: string) => void;

    let activeTab: "details" | "tags" = "details";
    let tagDraft = "";
    const progressStops = ["25%", "50%", "75%", "100%"];
    const statuses = ["Reading", "Queued", "Finished", "Archived"];

    function addDraftTag() {
        const tag = tagDraft.trim();
        if (!tag) return;
        onAddTag(book.id, tag);
        tagDraft = "";
    }
</script>

<aside class="pdf-inspector" aria-label="Selected PDF details">
    <header class="inspector-header">
        <div>
            <span>PDF</span>
            <h2>{book.title}</h2>
            <p>{book.creator}</p>
        </div>
        <button type="button" title="Open reader" aria-label="Open reader" onclick={() => onOpenReader(book.id)}>
            <BookOpen size={16} />
        </button>
    </header>

    <div class="inspector-preview" style={`--accent:${book.accent}`}>
        <div class="preview-cover"><span></span><strong>{book.type}</strong></div>
        <div class="preview-page">
            <span class="preview-line wide"></span><span class="preview-line"></span>
            <span class="preview-line short"></span><span class="preview-highlight"></span>
            <span class="preview-line"></span><span class="preview-line wide"></span>
        </div>
    </div>

    <nav class="inspector-tabs" aria-label="Inspector sections">
        <button type="button" class:active={activeTab === "details"} onclick={() => (activeTab = "details")}>Details</button>
        <button type="button" class:active={activeTab === "tags"} onclick={() => (activeTab = "tags")}>Tags</button>
    </nav>

    {#if activeTab === "details"}
        <section class="inspector-section" aria-label="PDF metadata">
            <p class="description">{book.description}</p>
            <dl class="metadata-grid">
                <div><dt>Status</dt><dd>{book.status}</dd></div>
                <div><dt>Progress</dt><dd>{book.progress} · p. {book.currentPage}</dd></div>
                <div><dt>Pages</dt><dd>{book.pages}</dd></div>
                <div><dt>Size</dt><dd>{book.fileSize}</dd></div>
                <div><dt>Folder</dt><dd>{book.source}</dd></div>
                <div><dt>Modified</dt><dd>{book.modifiedAt}</dd></div>
            </dl>
            <label class="path-field"><span>File path</span><input readonly value={book.filePath} /></label>
            <div class="metadata-row"><span>Fingerprint</span><code>{book.fingerprint}</code></div>
        </section>
    {:else}
        <section class="inspector-section" aria-label="PDF tags and reading state">
            <div class="tag-list">
                {#each book.tags as tag}
                    <button type="button" class="tag-chip" onclick={() => onRemoveTag(book.id, tag)} title={`Remove ${tag}`}>
                        <span>{tag}</span><X size={12} />
                    </button>
                {/each}
            </div>
            <div class="tag-editor">
                <input bind:value={tagDraft} placeholder="Add tag" aria-label="Add tag" onkeydown={(event) => event.key === "Enter" && addDraftTag()} />
                <button type="button" aria-label="Save tag" onclick={addDraftTag}><Tag size={14} /></button>
            </div>

            <div class="control-group">
                <h3>Progress</h3>
                <div class="segmented-row">
                    {#each progressStops as progress}
                        <button type="button" class:active={book.progress === progress} onclick={() => onSetProgress(book.id, progress)}>{progress}</button>
                    {/each}
                </div>
            </div>
            <div class="control-group">
                <h3>Status</h3>
                <div class="segmented-row stacked">
                    {#each statuses as status}
                        <button type="button" class:active={book.status === status} onclick={() => onSetStatus(book.id, status)}>{status}</button>
                    {/each}
                </div>
            </div>
        </section>
    {/if}
</aside>
