<script lang="ts">
  import { Download, RefreshCw, Search, Settings } from '@lucide/svelte';
  import type { AppAction } from '../lib/types';

  export let onAction: (action: AppAction) => void;
</script>

<header class="top-bar" aria-label="Application toolbar">
  <div class="brand-lockup">
    <strong>Librik</strong>
    <nav class="app-menu" aria-label="Application menu">
      <button type="button" onclick={() => onAction('Profile')}>File</button>
      <button type="button" onclick={() => onAction('Filter')}>Edit</button>
      <button type="button" onclick={() => onAction('List view')}>View</button>
    </nav>
  </div>

  <label class="global-search">
    <Search size={15} />
    <input
      aria-label="Search books, PDFs, notes, annotations, and tags"
      placeholder="Search PDFs, tags, notes"
      onclick={() => onAction('Filter')}
    />
  </label>

  <div class="top-actions" aria-label="Global actions">
    <button type="button" title="Import PDF" aria-label="Import PDF" onclick={() => onAction('Import PDF')}>
      <Download size={16} />
    </button>
    <button type="button" title="Sync" aria-label="Sync" onclick={() => onAction('Sync')}>
      <RefreshCw size={16} />
    </button>
    <button type="button" title="Settings" aria-label="Settings" onclick={() => onAction('Settings')}>
      <Settings size={16} />
    </button>
    <button type="button" class="profile-button" aria-label="Profile" onclick={() => onAction('Profile')}>AK</button>
  </div>
</header>
