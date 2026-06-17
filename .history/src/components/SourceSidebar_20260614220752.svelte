<script lang="ts">
  import {
    BookOpen,
    ChevronDown,
    ChevronRight,
    Copy,
    FileQuestion,
    FileX,
    Folder,
    FolderOpen,
    FolderPlus,
    Library,
    Newspaper,
    Rss,
    Search,
    Trash2,
    X
  } from '@lucide/svelte';
  import type { IconComponent, WorkspaceTab } from '../lib/types';
  import type { ActivityView } from './ActivityRail.svelte';

  export let selectedSource: string;
  export let vaultCount: string;
  export let tabs: WorkspaceTab[];
  export let activeTabId: string;
  export let activeActivity: ActivityView;
  export let onSelect: (source: string) => void;
  export let onSelectTab: (tabId: string) => void;
  export let onCloseTab: (tabId: string) => void;

  const tagShortcuts = ['distributed-systems', 'databases', 'ml', 'reference'];
  const recentShortcuts = ['Designing Data-Intensive Applications', 'Attention Is All You Need', 'Systems Performance'];

  type LibraryTreeNode = {
    id: string;
    label: string;
    selectValue: string;
    icon: IconComponent;
    count?: string;
    tone?: 'default' | 'special' | 'danger' | 'feed';
    children?: LibraryTreeNode[];
  };

  type VisibleTreeNode = {
    node: LibraryTreeNode;
    depth: number;
  };

  let expandedNodeIds = new Set(['root', 'dissertation', 'teaching', 'feeds', 'nyt']);

  const libraryTree: LibraryTreeNode[] = [
    {
      id: 'root',
      label: 'My Library',
      selectValue: 'All PDFs',
      icon: Library,
      count: vaultCount,
      children: [
        {
          id: 'depictions-media',
          label: 'Depictions In Media',
          selectValue: 'Depictions In Media',
          icon: Folder,
          count: '12'
        },
        {
          id: 'dissertation',
          label: 'Dissertation',
          selectValue: 'Dissertation',
          icon: Folder,
          count: '48',
          children: [
            { id: 'chapter-1', label: 'Chapter 1', selectValue: 'Chapter 1', icon: Folder, count: '8' },
            { id: 'chapter-2', label: 'Chapter 2', selectValue: 'Chapter 2', icon: Folder, count: '14' },
            { id: 'chapter-3', label: 'Chapter 3', selectValue: 'Chapter 3', icon: Folder, count: '16' },
            { id: 'chapter-4', label: 'Chapter 4', selectValue: 'Chapter 4', icon: Folder, count: '10' }
          ]
        },
        {
          id: 'teaching',
          label: 'Teaching',
          selectValue: 'Teaching',
          icon: Folder,
          count: '33',
          children: [
            { id: 'fall', label: 'Fall', selectValue: 'Fall', icon: Folder, count: '13' },
            { id: 'spring', label: 'Spring', selectValue: 'Spring', icon: Folder, count: '11' },
            { id: 'summer', label: 'Summer', selectValue: 'Summer', icon: Folder, count: '9' }
          ]
        },
        {
          id: 'duplicate-items',
          label: 'Duplicate Items',
          selectValue: 'Duplicate Items',
          icon: Copy,
          count: '3',
          tone: 'special'
        },
        {
          id: 'unfiled-items',
          label: 'Unfiled Items',
          selectValue: 'Unfiled Items',
          icon: FileQuestion,
          count: '7',
          tone: 'special'
        },
        {
          id: 'retracted-items',
          label: 'Retracted Items',
          selectValue: 'Retracted Items',
          icon: FileX,
          count: '1',
          tone: 'danger'
        },
        {
          id: 'trash',
          label: 'Trash',
          selectValue: 'Trash',
          icon: Trash2,
          tone: 'danger'
        }
      ]
    },
    {
      id: 'feeds',
      label: 'Feeds',
      selectValue: 'Feeds',
      icon: Rss,
      tone: 'feed',
      children: [
        {
          id: 'nyt',
          label: 'NYT',
          selectValue: 'NYT',
          icon: Rss,
          tone: 'feed',
          children: [
            {
              id: 'nyt-top-stories',
              label: 'Top Stories',
              selectValue: 'NYT > Top Stories',
              icon: Newspaper,
              count: '18',
              tone: 'feed'
            }
          ]
        }
      ]
    }
  ];

  $: panelTitle =
    activeActivity === 'search'
      ? 'Search'
      : activeActivity === 'tags'
        ? 'Tags'
        : activeActivity === 'recent'
          ? 'Recent'
          : activeActivity === 'settings'
            ? 'Settings'
            : 'Library';

  $: libraryTree[0].count = vaultCount;
  $: visibleTree = flattenTree(libraryTree, expandedNodeIds);

  function flattenTree(nodes: LibraryTreeNode[], expandedIds: Set<string>, depth = 0): VisibleTreeNode[] {
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
  <header class="side-panel-header">
    <span>{panelTitle}</span>
    <strong>Librik</strong>
  </header>

  <section class="source-section open-editors" aria-label="Open editors">
    <h2>Open Editors</h2>
    <div class="source-list editor-list">
      {#each tabs as tab}
        <div class="editor-row" class:active={activeTabId === tab.id}>
          <button type="button" onclick={() => onSelectTab(tab.id)}>
            {#if tab.kind === 'library'}
              <Library size={14} strokeWidth={1.8} />
            {:else}
              <BookOpen size={14} strokeWidth={1.8} />
            {/if}
            <span>{tab.title}</span>
          </button>
          {#if tab.closable}
            <button
              type="button"
              class="editor-close"
              title={`Close ${tab.title}`}
              aria-label={`Close ${tab.title}`}
              onclick={() => onCloseTab(tab.id)}
            >
              <X size={12} />
            </button>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  {#if activeActivity === 'search'}
    <section class="source-section activity-panel" aria-label="Search library">
      <h2>Search Library</h2>
      <label class="side-search">
        <span>Query</span>
        <input placeholder="PDFs, authors, notes" />
      </label>
      <button type="button" onclick={() => onSelect('All PDFs')}>All PDFs</button>
      <button type="button" onclick={() => onSelect('Annotated')}>Annotated PDFs</button>
      <button type="button" onclick={() => onSelect('Missing Files')}>Missing files</button>
    </section>
  {:else if activeActivity === 'tags'}
    <section class="source-section activity-panel" aria-label="Tag shortcuts">
      <h2>Tag Shortcuts</h2>
      {#each tagShortcuts as tag}
        <button type="button" onclick={() => onSelect(tag)}>#{tag}</button>
      {/each}
    </section>
  {:else if activeActivity === 'recent'}
    <section class="source-section activity-panel" aria-label="Recent documents">
      <h2>Recently Opened</h2>
      {#each recentShortcuts as title}
        <button type="button" onclick={() => onSelect('Recently Opened')}>{title}</button>
      {/each}
    </section>
  {:else if activeActivity === 'settings'}
    <section class="source-section activity-panel" aria-label="Workspace settings">
      <h2>Workspace</h2>
      <button type="button" onclick={() => onSelect('Vault settings')}>Vault folder</button>
      <button type="button" onclick={() => onSelect('Import behavior')}>Import behavior</button>
      <button type="button" onclick={() => onSelect('Tag filter')}>Reading defaults</button>
    </section>
  {/if}

  <section class="source-section library-tree-section" aria-label="Library tree">
    <div class="library-tree-toolbar" aria-label="Library actions">
      <button type="button" title="New collection" aria-label="New collection">
        <FolderPlus size={15} strokeWidth={1.8} />
      </button>
      <button type="button" title="Search collections" aria-label="Search collections">
        <Search size={15} strokeWidth={1.8} />
      </button>
    </div>

    <div class="vault-tree">
      {#each visibleTree as { node, depth }}
        <div
          class:tree-spacer={node.id === 'feeds'}
          class="tree-row-wrap"
          style={`--tree-depth: ${depth}`}
        >
          <button
            type="button"
            class="tree-row"
            class:active={isTreeNodeActive(node)}
            class:special={node.tone === 'special'}
            class:danger={node.tone === 'danger'}
            class:feed={node.tone === 'feed'}
            aria-pressed={isTreeNodeActive(node)}
            onclick={() => selectTreeNode(node)}
          >
            {#if node.children}
              <span
                class="tree-disclosure"
                role="button"
                tabindex="0"
                aria-label={isExpanded(node) ? `Collapse ${node.label}` : `Expand ${node.label}`}
                onclick={(event) => {
                  event.stopPropagation();
                  toggleTreeNode(node.id);
                }}
                onkeydown={(event) => {
                  if (event.key !== 'Enter' && event.key !== ' ') return;
                  event.preventDefault();
                  event.stopPropagation();
                  toggleTreeNode(node.id);
                }}
              >
                {#if isExpanded(node)}
                  <ChevronDown size={14} strokeWidth={1.8} />
                {:else}
                  <ChevronRight size={14} strokeWidth={1.8} />
                {/if}
              </span>
            {:else}
              <span class="tree-disclosure" aria-hidden="true"></span>
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
</aside>
