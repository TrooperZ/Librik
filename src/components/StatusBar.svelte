<script lang="ts">
  export let source: string;
  export let count: string;
  export let selectedTitle: string;
</script>

<footer class="status-bar" aria-label="Library status">
  <span>{source}</span>
  <span>{count} items</span>
  <span class="selected-title">{selectedTitle}</span>
  <span>Ready</span>
</footer>
