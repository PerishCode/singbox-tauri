<script lang="ts">
  export let items: Array<[string, string]> = [];
</script>

<div class="space-y-3">
  {#each items as [label, value]}
    <div class="value-card">
      <div class="value-label">{label}</div>
      <code class="value-code">{value}</code>
    </div>
  {/each}
</div>
