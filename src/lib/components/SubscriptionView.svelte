<script lang="ts">
  import JsonPreview from "./JsonPreview.svelte";
  import type { SubscriptionDefinitionSnapshot } from "../types";
  import Panel from "./Panel.svelte";

  export let subscription: SubscriptionDefinitionSnapshot | null = null;

  $: currentEntry = subscription
    ? {
        id: subscription.id ?? "unconfigured",
        label: subscription.label,
        type: subscription.type,
        scope: subscription.scope,
        profile: subscription.profile ?? "unknown",
        adapter: subscription.adapter,
        source: subscription.source,
      }
    : null;
</script>

<div class="space-y-6">
  <header class="panel space-y-3">
    <div class="badge preset-tonal-primary inline-flex">subscription</div>
    <h1 class="text-3xl font-semibold tracking-tight text-white">订阅定义</h1>
    <p class="max-w-3xl text-sm leading-6 text-slate-300">这里只保留订阅本身的定义。运行时事实统一交给观察者页面暴露。</p>
  </header>

  {#if subscription}
    <Panel title="Current Entry" badge={subscription.source.type ?? "none"}>
      <JsonPreview value={currentEntry} />
    </Panel>
  {:else}
    <section class="alert-error">Subscription state is unavailable.</section>
  {/if}

  {#if subscription?.entries?.length}
    <Panel title="Registry Entries" badge={String(subscription.entries.length)} badgeClass="badge preset-tonal-primary">
      <JsonPreview value={subscription.entries} />
    </Panel>
  {/if}
</div>
