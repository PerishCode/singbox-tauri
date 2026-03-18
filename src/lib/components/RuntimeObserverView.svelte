<script lang="ts">
  import type { SingboxRuntimeStatus } from "../api/generated";
  import type { SubscriptionRuntimeSnapshot } from "../types";
  import JsonPreview from "./JsonPreview.svelte";
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

  $: runtimeView = runtimeStatus
    ? {
        processStatus: runtimeStatus.processStatus,
        pid: runtimeStatus.pid ?? null,
        lifecycle: runtimeStatus.lifecycle,
        configPath: runtimeStatus.configPath,
        logPath: runtimeStatus.logPath,
      }
    : null;

  $: subscriptionView = subscriptionRuntime
    ? {
        subscriptionId: subscriptionRuntime.subscriptionId ?? "unbound",
        keyState: subscriptionRuntime.keyState,
        fetchState: subscriptionRuntime.fetchState,
        decryptState: subscriptionRuntime.decryptState,
        applyState: subscriptionRuntime.applyState,
        lastAttemptAt: formatTimestamp(subscriptionRuntime.lastAttemptAt),
        lastSuccessfulRefreshAt: formatTimestamp(subscriptionRuntime.lastSuccessfulRefreshAt),
        activeConfigPath: subscriptionRuntime.activeConfigPath,
      }
    : null;

  $: artifactView = subscriptionRuntime
    ? {
        privateKeyPath: subscriptionRuntime.privateKeyPath,
        publicKeyPath: subscriptionRuntime.publicKeyPath,
        encryptedPath: subscriptionRuntime.encryptedPath,
        decryptedPath: subscriptionRuntime.decryptedPath,
      }
    : null;
</script>

<div class="space-y-6">
  <header class="panel space-y-4">
    <div class="badge preset-tonal-primary inline-flex">runtime observer</div>
    <div class="space-y-3">
      <h1 class="text-3xl font-semibold tracking-tight text-white">运行时观察者</h1>
      <p class="max-w-3xl text-sm leading-6 text-slate-300">这里只暴露实际运行中的 sing-box 状态、订阅产物状态和应用结果。</p>
    </div>
    <div class="action-row">
      <button class="action-btn-ghost" disabled={refreshPending || applyPending} on:click={onRefresh}>
        {refreshPending ? "Refreshing..." : "Refresh Subscription"}
      </button>
      <button class="action-btn-primary" disabled={applyPending || refreshPending} on:click={onApply}>
        {applyPending ? "Applying..." : "Refresh and Apply"}
      </button>
    </div>
  </header>

  {#if refreshError}
    <section class="alert-error">{refreshError}</section>
  {/if}

  <section class="grid gap-6 xl:grid-cols-2">
    <Panel title="Sing-box Runtime" badge="live" badgeClass="badge preset-tonal-warning">
      <JsonPreview value={runtimeView} />
    </Panel>

    <Panel title="Subscription Runtime" badge="observer">
      <JsonPreview value={subscriptionView} />
    </Panel>
  </section>

  {#if subscriptionRuntime}
    <Panel title="Apply State" badge={subscriptionRuntime.applyState} badgeClass="badge preset-tonal-warning">
      <JsonPreview value={{ applyMessage: subscriptionRuntime.applyMessage }} />
    </Panel>

    <Panel title="Artifacts" badge="local files" badgeClass="badge preset-tonal-primary">
      <JsonPreview value={artifactView} />
    </Panel>

    <Panel title="Last Error" badge={subscriptionRuntime.lastError ? "present" : "clear"} badgeClass={subscriptionRuntime.lastError ? "badge preset-tonal-error" : "badge preset-tonal-surface"}>
      <JsonPreview value={{ lastError: subscriptionRuntime.lastError ?? "none" }} />
    </Panel>
  {/if}
</div>
