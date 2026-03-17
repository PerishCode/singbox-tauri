<script lang="ts">
  import type { SingboxRuntimeStatus } from "../api/generated";
  import type { SubscriptionRuntimeSnapshot } from "../types";
  import Panel from "./Panel.svelte";

  export let runtimeStatus: SingboxRuntimeStatus | null = null;
  export let subscriptionRuntime: SubscriptionRuntimeSnapshot | null = null;
  export let refreshPending = false;
  export let applyPending = false;
  export let refreshError: string | null = null;
  export let onRefresh: () => void | Promise<void>;
  export let onApply: () => void | Promise<void>;

  function formatTimestamp(value: string | null | undefined): string {
    if (!value) return "never";
    const unix = Number(value);
    if (Number.isNaN(unix)) return value;
    return new Date(unix * 1000).toLocaleString();
  }

  $: runtimeItems = runtimeStatus
    ? [
        ["process status", runtimeStatus.processStatus],
        ["pid", runtimeStatus.pid ? String(runtimeStatus.pid) : "none"],
        ["lifecycle", runtimeStatus.lifecycle],
        ["config", runtimeStatus.configPath],
        ["log", runtimeStatus.logPath],
      ]
    : [];

  $: observerItems = subscriptionRuntime
    ? [
        ["subscription id", subscriptionRuntime.subscriptionId ?? "unbound"],
        ["key state", subscriptionRuntime.keyState],
        ["fetch state", subscriptionRuntime.fetchState],
        ["decrypt state", subscriptionRuntime.decryptState],
        ["apply state", subscriptionRuntime.applyState],
        ["last attempt", formatTimestamp(subscriptionRuntime.lastAttemptAt)],
        ["last success", formatTimestamp(subscriptionRuntime.lastSuccessfulRefreshAt)],
        ["active config", subscriptionRuntime.activeConfigPath],
      ]
    : [];

  $: artifactItems = subscriptionRuntime
    ? [
        ["private key", subscriptionRuntime.privateKeyPath],
        ["public key", subscriptionRuntime.publicKeyPath],
        ["encrypted payload", subscriptionRuntime.encryptedPath],
        ["decrypted payload", subscriptionRuntime.decryptedPath],
      ]
    : [];
</script>

<div class="space-y-6">
  <section class="panel-hero">
    <p class="text-xs font-semibold uppercase tracking-[0.22em] text-sky-300/80">runtime observer</p>
    <h1 class="mt-2 text-3xl font-semibold tracking-tight text-white">运行时观察者</h1>
    <p class="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
      这里只暴露实际运行中的 sing-box 状态、订阅产物状态和应用结果，尽量避免覆盖式配置心智。
    </p>
    <div class="action-row">
      <button class="action-btn-ghost" disabled={refreshPending || applyPending} on:click={onRefresh}>
        {refreshPending ? "Refreshing..." : "Refresh Subscription"}
      </button>
      <button class="action-btn-primary" disabled={applyPending || refreshPending} on:click={onApply}>
        {applyPending ? "Applying..." : "Refresh and Apply"}
      </button>
    </div>
  </section>

  {#if refreshError}
    <section class="alert-error">{refreshError}</section>
  {/if}

  <section class="grid gap-6 xl:grid-cols-2">
    <Panel title="Sing-box Runtime" badge="live" badgeClass="badge-live">
      <div class="space-y-3">
        {#each runtimeItems as [label, value]}
          <div class="value-card">
            <div class="value-label">{label}</div>
            <code class="value-code">{value}</code>
          </div>
        {/each}
      </div>
    </Panel>

    <Panel title="Subscription Runtime" badge="observer" badgeClass="badge-subtle">
      <div class="space-y-3">
        {#each observerItems as [label, value]}
          <div class="value-card">
            <div class="value-label">{label}</div>
            <code class="value-code">{value}</code>
          </div>
        {/each}
      </div>
    </Panel>
  </section>

  {#if subscriptionRuntime}
    <Panel title="Apply State" badge={subscriptionRuntime.applyState} badgeClass="badge-live">
      <div class="value-card">
        <div class="value-label">runtime apply status</div>
        <div class="text-sm leading-6 text-slate-100">{subscriptionRuntime.applyMessage}</div>
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

    <Panel title="Last Error" badge={subscriptionRuntime.lastError ? "present" : "clear"} badgeClass="badge-live">
      <div class="value-card">
        <div class="value-label">subscription error</div>
        <code class="value-code">{subscriptionRuntime.lastError ?? "none"}</code>
      </div>
    </Panel>
  {/if}
</div>
