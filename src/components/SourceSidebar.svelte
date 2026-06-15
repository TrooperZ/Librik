<script lang="ts">
  import type { SourceSection } from '../lib/types';

  export let sections: SourceSection[];
  export let selectedSource: string;
  export let onSelect: (source: string) => void;
</script>

<aside class="source-sidebar" aria-label="Library sources">
  {#each sections as section}
    <section class="source-section" aria-label={section.label}>
      <h2>{section.label}</h2>
      <div class="source-list">
        {#each section.items as item}
          <button
            type="button"
            class:active={selectedSource === item.label}
            aria-pressed={selectedSource === item.label}
            onclick={() => onSelect(item.label)}
          >
            <item.icon size={15} strokeWidth={1.8} />
            <span>{item.label}</span>
            {#if item.count}
              <strong>{item.count}</strong>
            {/if}
          </button>
        {/each}
      </div>
    </section>
  {/each}

  <button type="button" class="tag-filter" onclick={() => onSelect('Tag filter')}>
    <span>Tag filter</span>
    <strong>Cmd K</strong>
  </button>
</aside>
