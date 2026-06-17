<script lang="ts">
    import {
        ChevronDown,
        ChevronRight,
        Copy,
        Filter,
        FileQuestion,
        FileX,
        Folder,
        FolderOpen,
        FolderPlus,
        Library,
        Plus,
        Search,
        Trash2,
    } from "@lucide/svelte";
    import type { IconComponent } from "../lib/types";
    import type { ActivityView } from "./ActivityRail.svelte";

    export let selectedSource: string;
    export let vaultCount: string;
    export let activeActivity: ActivityView;
    export let onSelect: (source: string) => void;

    const recentFiles = [
        {
            title: "Designing Data-Intensive Applications",
            meta: "Martin Kleppmann · Books",
        },
        {
            title: "Attention Is All You Need",
            meta: "Vaswani et al. · Research",
        },
        {
            title: "Systems Performance",
            meta: "Brendan Gregg · Manuals",
        },
        {
            title: "Engineering a Compiler",
            meta: "Cooper, Torczon · Textbooks",
        },
        {
            title: "Distributed Systems Reading Pack",
            meta: "Course Pack · Course Packs",
        },
    ];

    const tagList = [
        { label: "distributed-systems", count: 2, checked: true },
        { label: "databases", count: 1, checked: true },
        { label: "architecture", count: 1, checked: false },
        { label: "software-craft", count: 1, checked: false },
        { label: "performance", count: 1, checked: false },
        { label: "systems", count: 2, checked: false },
        { label: "reference", count: 1, checked: false },
        { label: "ml", count: 1, checked: false },
        { label: "transformers", count: 1, checked: false },
        { label: "compilers", count: 1, checked: false },
        { label: "textbook", count: 1, checked: false },
        { label: "course", count: 1, checked: false },
    ];

    type LibraryTreeNode = {
        id: string;
        label: string;
        selectValue: string;
        icon: IconComponent;
        count?: string;
        tone?: "default" | "special" | "danger" | "feed";
        children?: LibraryTreeNode[];
    };

    type VisibleTreeNode = {
        node: LibraryTreeNode;
        depth: number;
    };

    let expandedNodeIds = new Set(["root", "dissertation", "teaching"]);
    let tagSearchQuery = "";
    let showAddTagModal = false;

    const libraryTree: LibraryTreeNode[] = [
        {
            id: "root",
            label: "My Library",
            selectValue: "All PDFs",
            icon: Library,
            count: vaultCount,
            children: [
                {
                    id: "depictions-media",
                    label: "Depictions In Media",
                    selectValue: "Depictions In Media",
                    icon: Folder,
                    count: "12",
                },
                {
                    id: "dissertation",
                    label: "Dissertation",
                    selectValue: "Dissertation",
                    icon: Folder,
                    count: "48",
                    children: [
                        {
                            id: "chapter-1",
                            label: "Chapter 1",
                            selectValue: "Chapter 1",
                            icon: Folder,
                            count: "8",
                        },
                        {
                            id: "chapter-2",
                            label: "Chapter 2",
                            selectValue: "Chapter 2",
                            icon: Folder,
                            count: "14",
                        },
                        {
                            id: "chapter-3",
                            label: "Chapter 3",
                            selectValue: "Chapter 3",
                            icon: Folder,
                            count: "16",
                        },
                        {
                            id: "chapter-4",
                            label: "Chapter 4",
                            selectValue: "Chapter 4",
                            icon: Folder,
                            count: "10",
                        },
                    ],
                },
                {
                    id: "teaching",
                    label: "Teaching",
                    selectValue: "Teaching",
                    icon: Folder,
                    count: "33",
                    children: [
                        {
                            id: "fall",
                            label: "Fall",
                            selectValue: "Fall",
                            icon: Folder,
                            count: "13",
                        },
                        {
                            id: "spring",
                            label: "Spring",
                            selectValue: "Spring",
                            icon: Folder,
                            count: "11",
                        },
                        {
                            id: "summer",
                            label: "Summer",
                            selectValue: "Summer",
                            icon: Folder,
                            count: "9",
                        },
                    ],
                },
                {
                    id: "duplicate-items",
                    label: "Duplicate Items",
                    selectValue: "Duplicate Items",
                    icon: Copy,
                    count: "3",
                    tone: "special",
                },
                {
                    id: "unfiled-items",
                    label: "Unfiled Items",
                    selectValue: "Unfiled Items",
                    icon: FileQuestion,
                    count: "7",
                    tone: "special",
                },
                {
                    id: "retracted-items",
                    label: "Retracted Items",
                    selectValue: "Retracted Items",
                    icon: FileX,
                    count: "1",
                    tone: "danger",
                },
                {
                    id: "trash",
                    label: "Trash",
                    selectValue: "Trash",
                    icon: Trash2,
                    tone: "danger",
                },
            ],
        },
    ];

    $: libraryTree[0].count = vaultCount;
    $: visibleTree = flattenTree(libraryTree, expandedNodeIds);
    $: visibleTags = tagList.filter((tag) =>
        tag.label.toLowerCase().includes(tagSearchQuery.trim().toLowerCase()),
    );

    function flattenTree(
        nodes: LibraryTreeNode[],
        expandedIds: Set<string>,
        depth = 0,
    ): VisibleTreeNode[] {
        return nodes.flatMap((node) => {
            const row = { node, depth };
            if (!node.children || !expandedIds.has(node.id)) return [row];
            return [row, ...flattenTree(node.children, expandedIds, depth + 1)];
        });
    }

    function isExpanded(node: LibraryTreeNode) {
        return expandedNodeIds.has(node.id);
    }

    function isTreeNodeActive(node: LibraryTreeNode) {
        return selectedSource === node.selectValue;
    }

    function toggleTreeNode(nodeId: string) {
        const nextExpanded = new Set(expandedNodeIds);
        if (nextExpanded.has(nodeId)) {
            nextExpanded.delete(nodeId);
        } else {
            nextExpanded.add(nodeId);
        }
        expandedNodeIds = nextExpanded;
    }

    function selectTreeNode(node: LibraryTreeNode) {
        onSelect(node.selectValue);
    }
</script>

<aside class="source-sidebar" aria-label="Library sources">
    {#if activeActivity === "library"}
        <section
            class="source-section library-tree-section"
            aria-label="Library tree"
        >
            <header class="library-panel-header">
                <span>Library</span>
                <button
                    type="button"
                    title="New folder"
                    aria-label="New folder"
                >
                    <FolderPlus size={15} strokeWidth={1.8} />
                </button>
            </header>

            <div class="vault-tree">
                {#each visibleTree as { node, depth }}
                    <div
                        class:tree-spacer={node.id === "feeds"}
                        class="tree-row-wrap"
                        style={`--tree-depth: ${depth}`}
                    >
                        <button
                            type="button"
                            class="tree-row"
                            class:active={isTreeNodeActive(node)}
                            class:special={node.tone === "special"}
                            class:danger={node.tone === "danger"}
                            class:feed={node.tone === "feed"}
                            aria-pressed={isTreeNodeActive(node)}
                            onclick={() => selectTreeNode(node)}
                        >
                            {#if node.children}
                                <span
                                    class="tree-disclosure"
                                    role="button"
                                    tabindex="0"
                                    aria-label={isExpanded(node)
                                        ? `Collapse ${node.label}`
                                        : `Expand ${node.label}`}
                                    onclick={(event) => {
                                        event.stopPropagation();
                                        toggleTreeNode(node.id);
                                    }}
                                    onkeydown={(event) => {
                                        if (
                                            event.key !== "Enter" &&
                                            event.key !== " "
                                        )
                                            return;
                                        event.preventDefault();
                                        event.stopPropagation();
                                        toggleTreeNode(node.id);
                                    }}
                                >
                                    {#if isExpanded(node)}
                                        <ChevronDown
                                            size={14}
                                            strokeWidth={1.8}
                                        />
                                    {:else}
                                        <ChevronRight
                                            size={14}
                                            strokeWidth={1.8}
                                        />
                                    {/if}
                                </span>
                            {:else}
                                <span class="tree-disclosure" aria-hidden="true"
                                ></span>
                            {/if}

                            <span class="tree-icon">
                                {#if node.icon === Folder && node.children && isExpanded(node)}
                                    <FolderOpen size={16} strokeWidth={1.8} />
                                {:else}
                                    <node.icon size={16} strokeWidth={1.8} />
                                {/if}
                            </span>
                            <span>{node.label}</span>
                            {#if node.count}
                                <strong>{node.count}</strong>
                            {/if}
                        </button>
                    </div>
                {/each}
            </div>
        </section>
    {:else if activeActivity === "search"}
        <section class="activity-panel search-panel" aria-label="Search">
            <label class="compact-search">
                <Search size={15} strokeWidth={1.8} />
                <input placeholder="Search library" />
            </label>
            <button type="button" class="icon-square" aria-label="Filter">
                <Filter size={15} strokeWidth={1.8} />
            </button>
        </section>
    {:else if activeActivity === "tags"}
        <section class="tags-panel" aria-label="Tags">
            <div class="tag-search-row">
                <label class="compact-search">
                    <Search size={15} strokeWidth={1.8} />
                    <input
                        bind:value={tagSearchQuery}
                        placeholder="Search tags"
                    />
                </label>
                <button
                    type="button"
                    class="icon-square"
                    aria-label="Add tag"
                    onclick={() => {
                        showAddTagModal = true;
                    }}
                >
                    <Plus size={15} strokeWidth={1.8} />
                </button>
            </div>

            <div class="tag-check-list" aria-label="Tag list">
                {#each visibleTags as tag}
                    <label class="tag-check-row">
                        <input
                            type="checkbox"
                            checked={tag.checked}
                            onchange={() => onSelect(tag.label)}
                        />
                        <span>{tag.label}</span>
                        <strong>{tag.count}</strong>
                    </label>
                {/each}
            </div>

            {#if showAddTagModal}
                <div
                    class="mock-modal-backdrop"
                    role="presentation"
                    onclick={() => {
                        showAddTagModal = false;
                    }}
                >
                    <div
                        class="mock-modal"
                        role="dialog"
                        aria-modal="true"
                        aria-label="Add tag"
                        tabindex="-1"
                        onclick={(event) => event.stopPropagation()}
                        onkeydown={(event) => event.stopPropagation()}
                    >
                        <header>
                            <strong>Add Tag</strong>
                            <button
                                type="button"
                                aria-label="Close add tag"
                                onclick={() => {
                                    showAddTagModal = false;
                                }}
                            >
                                X
                            </button>
                        </header>
                        <label class="modal-field">
                            <span>Name</span>
                            <input placeholder="New tag name" />
                        </label>
                        <div class="modal-actions">
                            <button
                                type="button"
                                onclick={() => {
                                    showAddTagModal = false;
                                }}>Cancel</button
                            >
                            <button
                                type="button"
                                onclick={() => {
                                    showAddTagModal = false;
                                }}>Add</button
                            >
                        </div>
                    </div>
                </div>
            {/if}
        </section>
    {:else if activeActivity === "recent"}
        <section class="recent-panel" aria-label="Recently opened files">
            {#each recentFiles as file}
                <button
                    type="button"
                    class="recent-file-row"
                    onclick={() => onSelect("Recently Opened")}
                >
                    <span>{file.title}</span>
                    <small>{file.meta}</small>
                </button>
            {/each}
        </section>
    {:else if activeActivity === "settings"}
        <section
            class="source-section activity-panel"
            aria-label="Workspace settings"
        >
            <h2>Workspace</h2>
            <button type="button" onclick={() => onSelect("Vault settings")}
                >Vault folder</button
            >
            <button type="button" onclick={() => onSelect("Import behavior")}
                >Import behavior</button
            >
            <button type="button" onclick={() => onSelect("Tag filter")}
                >Reading defaults</button
            >
        </section>
    {/if}
</aside>
