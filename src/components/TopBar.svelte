<script lang="ts">
  import { Download, Moon, Search, Settings, Sun } from '@lucide/svelte';

  export let searchQuery: string;
  export let theme: 'light' | 'dark';
  export let onSearch: (query: string) => void;
  export let onImport: () => void;
  export let onSettings: () => void;
  export let onToggleTheme: () => void;
</script>

<header class="top-bar" aria-label="Application toolbar">
  <div class="brand-lockup">
    <strong>Librik</strong>
  </div>

  <label class="global-search">
    <Search size={15} />
    <input
      aria-label="Search library"
      placeholder="Search title, creator, description, or tag"
      value={searchQuery}
      oninput={(event) => onSearch(event.currentTarget.value)}
    />
  </label>

  <div class="top-actions" aria-label="Global actions">
    <button
      type="button"
      title={`Use ${theme === 'dark' ? 'light' : 'dark'} theme`}
      aria-label={`Use ${theme === 'dark' ? 'light' : 'dark'} theme`}
      aria-pressed={theme === 'dark'}
      onclick={onToggleTheme}
    >
      {#if theme === 'dark'}<Sun size={16} />{:else}<Moon size={16} />{/if}
    </button>
    <button type="button" title="Import PDF" aria-label="Import PDF" onclick={onImport}>
      <Download size={16} />
    </button>
    <button type="button" title="Settings" aria-label="Settings" onclick={onSettings}>
      <Settings size={16} />
    </button>
  </div>
</header>
