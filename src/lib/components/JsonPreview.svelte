<script lang="ts">
  export let value: unknown;
  export let emptyText = "<empty>";

  function format(input: unknown): string {
    if (input === null || input === undefined) return "";
    if (typeof input === "string") return input;
    try {
      return JSON.stringify(input, null, 2);
    } catch {
      return String(input);
    }
  }

  $: content = format(value);
</script>

<pre class="log-panel">{content || emptyText}</pre>
