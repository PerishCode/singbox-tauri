<script lang="ts">
  import RuntimeObserverView from "../lib/components/RuntimeObserverView.svelte";
  import type { SingboxRuntimeStatus } from "../lib/api/generated";
  import type { SubscriptionRuntimeSnapshot } from "../lib/types";

  export let runtimeStatus: SingboxRuntimeStatus | null = null;
  export let subscriptionRuntime: SubscriptionRuntimeSnapshot | null = null;
  export let refreshPending = false;
  export let applyPending = false;
  export let refreshError: string | null = null;
  export let onRefresh: () => void | Promise<void>;
  export let onApply: () => void | Promise<void>;
</script>

<RuntimeObserverView
  {runtimeStatus}
  {subscriptionRuntime}
  {refreshPending}
  {applyPending}
  {refreshError}
  {onRefresh}
  {onApply}
/>
