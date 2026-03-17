<script lang="ts">
  import type { SubscriptionSnapshot } from "../types";
  import Panel from "./Panel.svelte";

  export let subscription: SubscriptionSnapshot | null = null;

  $: summaryItems = subscription
    ? [
        ["key state", subscription.keyState],
        ["fetch state", subscription.fetchState],
        ["decrypt state", subscription.decryptState],
        ["source url", subscription.sourceUrl ?? "not configured"],
        ["public key", subscription.publicKey ?? "missing"],
        ["active config", subscription.activeConfigPath],
      ]
    : [];

  $: artifactItems = subscription
    ? [
        ["private key", subscription.privateKeyPath],
        ["public key path", subscription.publicKeyPath],
        ["encrypted payload", subscription.encryptedPath],
        ["decrypted payload", subscription.decryptedPath],
      ]
    : [];
</script>

<div class="space-y-6">
  <section class="panel-hero">
    <p class="text-xs font-semibold uppercase tracking-[0.22em] text-sky-300/80">subscription</p>
    <h1 class="mt-2 text-3xl font-semibold tracking-tight text-white">加密订阅状态</h1>
    <p class="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
      这里单独展示本地 age 密钥、远端密文拉取、解密结果和当前激活配置，避免把订阅主线混在概览页里。
    </p>
  </section>

  {#if subscription}
    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Readiness" badge={subscription.decryptState} badgeClass="badge-subtle">
        <div class="space-y-3">
          {#each summaryItems as [label, value]}
            <div class="value-card">
              <div class="value-label">{label}</div>
              <code class="value-code">{value}</code>
            </div>
          {/each}
        </div>
      </Panel>

      <Panel title="Artifacts" badge="local files" badgeClass="badge-readable">
        <div class="space-y-3">
          {#each artifactItems as [label, value]}
            <div class="value-card">
              <div class="value-label">{label}</div>
              <code class="value-code">{value}</code>
            </div>
          {/each}
        </div>
      </Panel>
    </section>

    <Panel title="Last Error" badge={subscription.lastError ? "present" : "clear"} badgeClass="badge-live">
      <div class="value-card">
        <div class="value-label">subscription error</div>
        <code class="value-code">{subscription.lastError ?? "none"}</code>
      </div>
    </Panel>
  {:else}
    <section class="alert-error">
      Subscription state is unavailable.
    </section>
  {/if}
</div>
