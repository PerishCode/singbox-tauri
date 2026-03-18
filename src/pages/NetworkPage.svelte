<script lang="ts">
  import LocalNetworkView from "../lib/components/LocalNetworkView.svelte";
  import type { LocalNetworkSnapshot } from "../lib/types";

  export let localNetwork: LocalNetworkSnapshot | null = null;
</script>

<LocalNetworkView network={localNetwork} />
