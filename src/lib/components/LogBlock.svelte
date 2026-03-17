<script lang="ts">
  export let content = "";
  export let emptyText = "<empty>";
</script>

<pre class="log-panel">{content || emptyText}</pre>
