<script lang="ts">
  import SubscriptionView from "../lib/components/SubscriptionView.svelte";
  import type { SubscriptionDefinitionSnapshot } from "../lib/types";

  export let subscription: SubscriptionDefinitionSnapshot | null = null;
</script>

<SubscriptionView {subscription} />
