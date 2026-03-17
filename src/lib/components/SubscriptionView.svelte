<script lang="ts">
  import type { SubscriptionDefinitionSnapshot } from "../types";
  import Panel from "./Panel.svelte";

  export let subscription: SubscriptionDefinitionSnapshot | null = null;

  $: summaryItems = subscription
    ? [
        ["id", subscription.id ?? "unconfigured"],
        ["label", subscription.label],
        ["type", subscription.type],
        ["scope", subscription.scope],
        ["profile", subscription.profile ?? "unknown"],
        ["adapter", subscription.adapter],
        ["source type", subscription.source.type ?? "none"],
        ["source url", subscription.source.url ?? "not configured"],
        ["source path", subscription.source.path ?? "not configured"],
      ]
    : [];
</script>

<div class="space-y-6">
  <section class="panel-hero">
    <p class="text-xs font-semibold uppercase tracking-[0.22em] text-sky-300/80">subscription</p>
    <h1 class="mt-2 text-3xl font-semibold tracking-tight text-white">订阅定义</h1>
    <p class="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
      这里只保留订阅本身的定义：条目 id、label、source 和 adapter。运行时事实统一交给观察者页面暴露。
    </p>
  </section>

  {#if subscription}
    <Panel title="Current Entry" badge={subscription.source.type ?? "none"} badgeClass="badge-subtle">
      <div class="space-y-3">
        {#each summaryItems as [label, value]}
          <div class="value-card">
            <div class="value-label">{label}</div>
            <code class="value-code">{value}</code>
          </div>
        {/each}
      </div>
    </Panel>
  {:else}
    <section class="alert-error">
      Subscription state is unavailable.
    </section>
  {/if}
</div>
