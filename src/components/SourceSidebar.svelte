<script lang="ts">
    import {
        ChevronDown,
        ChevronRight,
        Folder,
        FolderPlus,
        MoreHorizontal,
        Pencil,
        Search,
        Trash2,
    } from "@lucide/svelte";
    import { tick } from "svelte";
    import type { BookRow, FolderRow } from "../lib/types";
    import type { ActivityView } from "./ActivityRail.svelte";

    export let books: BookRow[];
    export let folders: FolderRow[];
    export let activeActivity: ActivityView;
    export let selectedFilter: string;
    export let selectedTag: string;
    export let selectedFolder: string;
    export let searchQuery: string;
    export let onFilter: (filter: string) => void;
    export let onSearch: (query: string) => void;
    export let onTag: (tag: string) => void;
    export let onFolder: (folderId: string) => void;
    export let onCreateFolder: (
        name: string,
        parentId: string | null,
    ) => Promise<FolderRow | null>;
    export let onRenameFolder: (id: string, name: string) => Promise<boolean>;
    export let onMoveFolder: (
        id: string,
        parentId: string | null,
        beforeId: string | null,
    ) => Promise<boolean>;
    export let onDeleteFolder: (id: string) => Promise<boolean>;
    export let onSetFolderColor: (
        id: string,
        color: string | null,
    ) => Promise<boolean>;
    export let onOpenBook: (bookId: string) => void;

    type VisibleFolder = {
        folder: FolderRow;
        depth: number;
        hasChildren: boolean;
    };
    type DropMode = "before" | "inside" | "after";

    const folderColors = [
        "#4f7cff",
        "#8b5cf6",
        "#db4b9b",
        "#e25555",
        "#e58a2f",
        "#2ca36c",
        "#2b9eb3",
    ];
    let tagSearchQuery = "";
    let expandedFolderIds = new Set<string>();
    let knownFolderIds = "";
    let creatingParentId: string | null | undefined;
    let renamingFolderId = "";
    let folderDraft = "";
    let menuFolderId = "";
    let draggedFolderId = "";
    let dropTargetId = "";
    let dropMode: DropMode | "" = "";
    let pointerFolderId = "";
    let pointerStartX = 0;
    let pointerStartY = 0;
    let suppressFolderClick = false;
    let editInput: HTMLInputElement;

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
            count: books.filter((book) => book.status === "Missing file")
                .length,
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
    $: folderIds = folders
        .map((folder) => folder.id)
        .sort()
        .join(":");
    $: if (folderIds !== knownFolderIds) {
        knownFolderIds = folderIds;
        expandedFolderIds = new Set([
            ...expandedFolderIds,
            ...folders.flatMap((folder) =>
                folder.parentId ? [folder.parentId] : [],
            ),
        ]);
    }
    $: visibleFolders = flattenFolders(folders, null, expandedFolderIds);

    function flattenFolders(
        allFolders: FolderRow[],
        parentId: string | null,
        expandedIds: Set<string>,
        depth = 0,
    ): VisibleFolder[] {
        return allFolders
            .filter((folder) => folder.parentId === parentId)
            .sort(
                (left, right) =>
                    left.sortOrder - right.sortOrder ||
                    left.name.localeCompare(right.name),
            )
            .flatMap((folder) => {
                const hasChildren = allFolders.some(
                    (candidate) => candidate.parentId === folder.id,
                );
                const row = { folder, depth, hasChildren };
                return hasChildren && expandedIds.has(folder.id)
                    ? [
                          row,
                          ...flattenFolders(
                              allFolders,
                              folder.id,
                              expandedIds,
                              depth + 1,
                          ),
                      ]
                    : [row];
            });
    }

    function toggleFolder(folderId: string) {
        const next = new Set(expandedFolderIds);
        next.has(folderId) ? next.delete(folderId) : next.add(folderId);
        expandedFolderIds = next;
    }

    async function focusEditor() {
        await tick();
        editInput?.focus();
        editInput?.select();
    }

    function startCreate(parentId: string | null) {
        menuFolderId = "";
        creatingParentId = parentId;
        renamingFolderId = "";
        folderDraft = "";
        if (parentId) {
            expandedFolderIds = new Set([...expandedFolderIds, parentId]);
        }
        void focusEditor();
    }

    async function finishCreate() {
        const parentId = creatingParentId;
        if (parentId === undefined) return;
        const name = folderDraft.trim();
        creatingParentId = undefined;
        if (!name) return;
        const created = await onCreateFolder(name, parentId);
        if (!created) {
            creatingParentId = parentId;
            folderDraft = name;
            void focusEditor();
        }
    }

    function startRename(folder: FolderRow) {
        menuFolderId = "";
        creatingParentId = undefined;
        renamingFolderId = folder.id;
        folderDraft = folder.name;
        void focusEditor();
    }

    async function finishRename(folder: FolderRow) {
        if (renamingFolderId !== folder.id) return;
        const name = folderDraft.trim();
        renamingFolderId = "";
        if (!name || name === folder.name) return;
        if (!(await onRenameFolder(folder.id, name))) {
            startRename(folder);
        }
    }

    async function deleteFolder(folder: FolderRow) {
        menuFolderId = "";
        if (
            !window.confirm(
                `Delete “${folder.name}” and its subfolders? PDFs will become unfiled.`,
            )
        )
            return;
        await onDeleteFolder(folder.id);
    }

    async function colorFolder(folder: FolderRow, color: string | null) {
        if (await onSetFolderColor(folder.id, color)) menuFolderId = "";
    }

    function canDropFolder(parentId: string | null) {
        const dragged = folders.find((folder) => folder.id === draggedFolderId);
        if (!dragged || dragged.id === parentId) return false;
        for (let id = parentId; id; ) {
            if (id === dragged.id) return false;
            id = folders.find((folder) => folder.id === id)?.parentId ?? null;
        }
        return true;
    }

    function placementFor(folder: FolderRow, mode: DropMode) {
        if (folder.id === draggedFolderId) return null;
        if (mode === "inside") {
            return canDropFolder(folder.id)
                ? { parentId: folder.id, beforeId: null }
                : null;
        }
        if (!canDropFolder(folder.parentId)) return null;
        const siblings = folders
            .filter(
                (candidate) =>
                    candidate.parentId === folder.parentId &&
                    candidate.id !== draggedFolderId,
            )
            .sort(
                (left, right) =>
                    left.sortOrder - right.sortOrder ||
                    left.name.localeCompare(right.name),
            );
        const targetIndex = siblings.findIndex(
            (candidate) => candidate.id === folder.id,
        );
        return {
            parentId: folder.parentId,
            beforeId:
                mode === "before"
                    ? folder.id
                    : (siblings[targetIndex + 1]?.id ?? null),
        };
    }

    function dragMode(clientY: number, row: HTMLElement): DropMode {
        const bounds = row.getBoundingClientRect();
        const ratio = (clientY - bounds.top) / bounds.height;
        return ratio < 0.25 ? "before" : ratio > 0.75 ? "after" : "inside";
    }

    function startFolderPointer(event: PointerEvent, folder: FolderRow) {
        if (
            event.button !== 0 ||
            (event.target as HTMLElement).closest("button, input")
        )
            return;
        pointerFolderId = folder.id;
        pointerStartX = event.clientX;
        pointerStartY = event.clientY;
    }

    function trackFolderPointer(event: PointerEvent) {
        if (!pointerFolderId) return;
        if (
            !draggedFolderId &&
            Math.hypot(
                event.clientX - pointerStartX,
                event.clientY - pointerStartY,
            ) < 4
        )
            return;
        draggedFolderId = pointerFolderId;
        event.preventDefault();
        const element = document.elementFromPoint(
            event.clientX,
            event.clientY,
        ) as HTMLElement | null;
        if (element?.closest("[data-folder-root]")) {
            const dragged = folders.find(
                (folder) => folder.id === draggedFolderId,
            );
            if (dragged?.parentId !== null) {
                dropTargetId = "root";
                dropMode = "inside";
                return;
            }
        }
        const row = element?.closest<HTMLElement>("[data-folder-id]");
        const folder = folders.find(
            (candidate) => candidate.id === row?.dataset.folderId,
        );
        if (row && folder) {
            const mode = dragMode(event.clientY, row);
            if (placementFor(folder, mode)) {
                dropTargetId = folder.id;
                dropMode = mode;
                return;
            }
        }
        dropTargetId = "";
        dropMode = "";
    }

    async function finishFolderPointer() {
        const id = draggedFolderId;
        const targetId = dropTargetId;
        const mode = dropMode;
        const folder = folders.find((candidate) => candidate.id === targetId);
        const placement = folder && mode ? placementFor(folder, mode) : null;
        pointerFolderId = "";
        clearFolderDrag();
        if (!id) return;
        suppressFolderClick = true;
        setTimeout(() => (suppressFolderClick = false));
        if (targetId === "root") {
            await onMoveFolder(id, null, null);
        } else if (folder && mode && placement) {
            if (
                await onMoveFolder(id, placement.parentId, placement.beforeId)
            ) {
                if (mode === "inside") {
                    expandedFolderIds = new Set([
                        ...expandedFolderIds,
                        folder.id,
                    ]);
                }
            }
        }
    }

    function clearFolderDrag() {
        draggedFolderId = "";
        dropTargetId = "";
        dropMode = "";
    }
</script>

<svelte:window
    onpointermove={trackFolderPointer}
    onpointerup={() => void finishFolderPointer()}
    onpointercancel={() => {
        pointerFolderId = "";
        clearFolderDrag();
    }}
/>

<aside class="source-sidebar" aria-label="Library sources">
    {#if activeActivity === "library"}
        <section
            class="source-section library-tree-section"
            aria-label="Library filters"
        >
            <header class="library-panel-header">
                <span>Library</span>
                <button
                    type="button"
                    title="New folder"
                    aria-label="New folder"
                    onclick={() => startCreate(null)}
                >
                    <FolderPlus size={15} strokeWidth={1.8} />
                </button>
            </header>
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
                        <Folder size={16} strokeWidth={1.8} />
                        <span>{filter.label}</span>
                        <strong class="tree-row-count">{filter.count}</strong>
                    </button>
                {/each}

                <div
                    class="folder-heading"
                    class:drop-inside={dropTargetId === "root"}
                    role="group"
                    aria-label="Collections root drop target"
                    data-folder-root
                >
                    Collections
                </div>
                {#if creatingParentId === null}
                    <div class="folder-edit-row" style="--folder-depth: 0">
                        <span></span><Folder size={16} />
                        <input
                            bind:this={editInput}
                            bind:value={folderDraft}
                            maxlength="64"
                            aria-label="New folder name"
                            onblur={finishCreate}
                            onkeydown={(event) => {
                                if (event.key === "Enter") void finishCreate();
                                if (event.key === "Escape")
                                    creatingParentId = undefined;
                            }}
                        />
                    </div>
                {/if}

                {#each visibleFolders as { folder, depth, hasChildren }}
                    <div
                        class="folder-tree-row"
                        class:active={selectedFolder === folder.id}
                        class:dragging={draggedFolderId === folder.id}
                        class:drop-before={dropTargetId === folder.id &&
                            dropMode === "before"}
                        class:drop-inside={dropTargetId === folder.id &&
                            dropMode === "inside"}
                        class:drop-after={dropTargetId === folder.id &&
                            dropMode === "after"}
                        class:folder-draggable={renamingFolderId !== folder.id}
                        style={`--folder-depth:${depth}; --folder-color:${folder.color ?? "var(--accent)"}`}
                        role="treeitem"
                        aria-selected={selectedFolder === folder.id}
                        tabindex="0"
                        data-folder-id={folder.id}
                        onpointerdown={(event) =>
                            startFolderPointer(event, folder)}
                        onclick={() => {
                            if (!suppressFolderClick) onFolder(folder.id);
                        }}
                        oncontextmenu={(event) => {
                            event.preventDefault();
                            menuFolderId = folder.id;
                        }}
                        onkeydown={(event) => {
                            if (event.key === "Enter" || event.key === " ")
                                onFolder(folder.id);
                        }}
                    >
                        {#if hasChildren}
                            <button
                                type="button"
                                class="folder-disclosure"
                                aria-label={expandedFolderIds.has(folder.id)
                                    ? `Collapse ${folder.name}`
                                    : `Expand ${folder.name}`}
                                onclick={(event) => {
                                    event.stopPropagation();
                                    toggleFolder(folder.id);
                                }}
                            >
                                {#if expandedFolderIds.has(folder.id)}<ChevronDown
                                        size={13}
                                    />{:else}<ChevronRight size={13} />{/if}
                            </button>
                        {:else}
                            <span></span>
                        {/if}
                        <Folder
                            class="folder-color-icon"
                            size={16}
                            strokeWidth={1.8}
                        />
                        {#if renamingFolderId === folder.id}
                            <input
                                bind:this={editInput}
                                bind:value={folderDraft}
                                maxlength="64"
                                aria-label={`Rename ${folder.name}`}
                                onclick={(event) => event.stopPropagation()}
                                onblur={() => finishRename(folder)}
                                onkeydown={(event) => {
                                    event.stopPropagation();
                                    if (event.key === "Enter")
                                        void finishRename(folder);
                                    if (event.key === "Escape")
                                        renamingFolderId = "";
                                }}
                            />
                        {:else}
                            <span class="folder-name">{folder.name}</span>
                        {/if}
                        <strong>{folder.documentCount}</strong>
                        <button
                            type="button"
                            class="folder-more"
                            title={`Folder actions for ${folder.name}`}
                            aria-label={`Folder actions for ${folder.name}`}
                            onclick={(event) => {
                                event.stopPropagation();
                                menuFolderId =
                                    menuFolderId === folder.id ? "" : folder.id;
                            }}><MoreHorizontal size={14} /></button
                        >
                    </div>

                    {#if menuFolderId === folder.id}
                        <div
                            class="folder-action-menu"
                            style={`--folder-depth:${depth}`}
                        >
                            <button
                                type="button"
                                onclick={() => startCreate(folder.id)}
                                ><FolderPlus size={13} />New subfolder</button
                            >
                            <button
                                type="button"
                                onclick={() => startRename(folder)}
                                ><Pencil size={13} />Rename</button
                            >
                            <div
                                class="folder-color-row"
                                aria-label="Folder color"
                            >
                                <button
                                    type="button"
                                    class="color-none"
                                    title="Default color"
                                    aria-label="Default color"
                                    onclick={() => colorFolder(folder, null)}
                                    >×</button
                                >
                                {#each folderColors as color}
                                    <button
                                        type="button"
                                        class="color-dot"
                                        class:active={folder.color === color}
                                        style={`--swatch:${color}`}
                                        title={color}
                                        aria-label={`Use ${color}`}
                                        onclick={() =>
                                            colorFolder(folder, color)}
                                    ></button>
                                {/each}
                            </div>
                            <button
                                type="button"
                                class="danger"
                                onclick={() => deleteFolder(folder)}
                                ><Trash2 size={13} />Delete</button
                            >
                        </div>
                    {/if}

                    {#if creatingParentId === folder.id}
                        <div
                            class="folder-edit-row"
                            style={`--folder-depth:${depth + 1}`}
                        >
                            <span></span><Folder size={16} />
                            <input
                                bind:this={editInput}
                                bind:value={folderDraft}
                                maxlength="64"
                                aria-label="New subfolder name"
                                onblur={finishCreate}
                                onkeydown={(event) => {
                                    if (event.key === "Enter")
                                        void finishCreate();
                                    if (event.key === "Escape")
                                        creatingParentId = undefined;
                                }}
                            />
                        </div>
                    {/if}
                {/each}
            </div>
        </section>
    {:else if activeActivity === "search"}
        <section
            class="activity-panel search-panel"
            aria-label="Search library"
        >
            <label class="compact-search"
                ><Search size={15} strokeWidth={1.8} /><input
                    value={searchQuery}
                    placeholder="Title, author, description, or tag"
                    oninput={(event) => onSearch(event.currentTarget.value)}
                /></label
            >
        </section>
    {:else if activeActivity === "tags"}
        <section class="tags-panel" aria-label="Tags">
            <div class="tag-search-row">
                <label class="compact-search"
                    ><Search size={15} strokeWidth={1.8} /><input
                        bind:value={tagSearchQuery}
                        placeholder="Search tags"
                    /></label
                >
            </div>
            <div class="tag-check-list" aria-label="Tag list">
                {#if tags.length === 0}<p class="empty-filter">
                        No tags yet.
                    </p>{/if}
                {#each tags as [tag, count]}
                    <button
                        type="button"
                        class="tag-check-row"
                        class:active={selectedTag === tag}
                        aria-pressed={selectedTag === tag}
                        onclick={() => onTag(selectedTag === tag ? "" : tag)}
                        ><span>{tag}</span><strong>{count}</strong></button
                    >
                {/each}
            </div>
        </section>
    {:else if activeActivity === "recent"}
        <section class="recent-panel" aria-label="Recently imported files">
            {#if books.length === 0}<p class="empty-filter">
                    No imported PDFs yet.
                </p>{/if}
            {#each books.slice(0, 10) as book}
                <button
                    type="button"
                    class="recent-file-row"
                    onclick={() => onOpenBook(book.id)}
                    ><span>{book.title}</span><small
                        >{book.creator} · {book.addedAt}</small
                    ></button
                >
            {/each}
        </section>
    {:else if activeActivity === "settings"}
        <section
            class="source-section activity-panel"
            aria-label="Workspace settings"
        >
            <h2>Workspace</h2>
            <dl class="workspace-facts">
                <div>
                    <dt>Storage</dt>
                    <dd>Managed vault</dd>
                </div>
                <div>
                    <dt>Documents</dt>
                    <dd>{books.length}</dd>
                </div>
                <div>
                    <dt>Database</dt>
                    <dd>SQLite schema v4</dd>
                </div>
            </dl>
        </section>
    {/if}
</aside>
