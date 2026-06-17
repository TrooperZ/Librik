<script lang="ts">
  import { BookOpen, Library, Plus, X } from '@lucide/svelte';
  import type { WorkspaceTab } from '../lib/types';

  export let tabs: WorkspaceTab[];
  export let activeTabId: string;
  export let onSelect: (tabId: string) => void;
  export let onClose: (tabId: string) => void;
  export let onNew: () => void;
  export let onReorder: (fromTabId: string, toTabId: string) => void;

  let draggedTabId: string | null = null;
  let dropTargetTabId: string | null = null;

  function isPinnedTab(tab: WorkspaceTab) {
    return tab.kind === 'library';
  }

  function handleDragStart(event: DragEvent, tabId: string) {
    const tab = tabs.find((item) => item.id === tabId);
    if (!tab || isPinnedTab(tab)) {
      event.preventDefault();
      return;
    }

    draggedTabId = tabId;
    dropTargetTabId = tabId;
    event.dataTransfer?.setData('text/plain', tabId);
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragEnter(event: DragEvent, tabId: string) {
    const tab = tabs.find((item) => item.id === tabId);
    if (!tab || isPinnedTab(tab) || !draggedTabId || draggedTabId === tabId) return;
    event.preventDefault();
    dropTargetTabId = tabId;
  }

  function handleDragOver(event: DragEvent, tabId: string) {
    const tab = tabs.find((item) => item.id === tabId);
    if (!tab || isPinnedTab(tab) || !draggedTabId || draggedTabId === tabId) return;
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    dropTargetTabId = tabId;
  }

  function handleDrop(event: DragEvent, tabId: string) {
    const tab = tabs.find((item) => item.id === tabId);
    if (!tab || isPinnedTab(tab)) {
      event.preventDefault();
      clearDragState();
      return;
    }

    event.preventDefault();
    const fromTabId = draggedTabId ?? event.dataTransfer?.getData('text/plain');
    draggedTabId = null;
    dropTargetTabId = null;
    if (!fromTabId || fromTabId === tabId) return;
    onReorder(fromTabId, tabId);
  }

  function clearDragState() {
    draggedTabId = null;
    dropTargetTabId = null;
  }

  function handleTabKeydown(event: KeyboardEvent, tabId: string) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onSelect(tabId);
  }
</script>

<nav class="workspace-tabs" aria-label="Workspace tabs">
  <div class="tab-strip" role="tablist" aria-label="Open workspaces">
    {#each tabs as tab}
      <div
        class="tab-item"
        class:active={activeTabId === tab.id}
        class:dragging={draggedTabId === tab.id}
        class:drop-target={dropTargetTabId === tab.id && draggedTabId !== tab.id}
        class:pinned={isPinnedTab(tab)}
        role="tab"
        aria-selected={activeTabId === tab.id}
        tabindex={activeTabId === tab.id ? 0 : -1}
        draggable={!isPinnedTab(tab)}
        ondragstart={(event) => handleDragStart(event, tab.id)}
        ondragenter={(event) => handleDragEnter(event, tab.id)}
        ondragover={(event) => handleDragOver(event, tab.id)}
        ondrop={(event) => handleDrop(event, tab.id)}
        ondragend={clearDragState}
        onkeydown={(event) => handleTabKeydown(event, tab.id)}
      >
        <button
          type="button"
          class="workspace-tab"
          onclick={() => onSelect(tab.id)}
        >
          {#if tab.kind === 'library'}
            <Library size={14} />
          {:else}
            <BookOpen size={14} />
          {/if}
          <span>{tab.title}</span>
        </button>

        {#if tab.closable}
          <button
            type="button"
            class="close-tab"
            title={`Close ${tab.title}`}
            aria-label={`Close ${tab.title}`}
            onclick={(event) => {
              event.stopPropagation();
              onClose(tab.id);
            }}
          >
            <X size={13} />
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <button type="button" class="new-tab" title="New PDF tab" aria-label="New PDF tab" onclick={onNew}>
    <Plus size={15} />
  </button>
</nav>
