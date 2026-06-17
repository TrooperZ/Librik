<script lang="ts">
  import { Clock3, Library, Search, Settings, Tag } from '@lucide/svelte';

  export type ActivityView = 'library' | 'search' | 'tags' | 'recent' | 'settings';

  export let activeView: ActivityView;
  export let onSelect: (view: ActivityView) => void;

  const primaryItems: Array<{ id: ActivityView; label: string; icon: typeof Library }> = [
    { id: 'library', label: 'Library', icon: Library },
    { id: 'search', label: 'Search', icon: Search },
    { id: 'tags', label: 'Tags', icon: Tag },
    { id: 'recent', label: 'Recent', icon: Clock3 }
  ];
</script>

<aside class="activity-rail" aria-label="Primary activity">
  <div class="activity-group">
    {#each primaryItems as item}
      <button
        type="button"
        class:active={activeView === item.id}
        title={item.label}
        aria-label={item.label}
        aria-pressed={activeView === item.id}
        onclick={() => onSelect(item.id)}
      >
        <item.icon size={20} strokeWidth={1.8} />
      </button>
    {/each}
  </div>

  <button
    type="button"
    class:active={activeView === 'settings'}
    title="Settings"
    aria-label="Settings"
    aria-pressed={activeView === 'settings'}
    onclick={() => onSelect('settings')}
  >
    <Settings size={20} strokeWidth={1.8} />
  </button>
</aside>
