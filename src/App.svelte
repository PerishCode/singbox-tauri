<script lang="ts">
  import { onMount } from "svelte";
  import KeyValueList from "./lib/components/KeyValueList.svelte";
  import LocalNetworkView from "./lib/components/LocalNetworkView.svelte";
  import LogBlock from "./lib/components/LogBlock.svelte";
  import Panel from "./lib/components/Panel.svelte";
  import HeroHeader from "./lib/components/HeroHeader.svelte";
  import RuntimeObserverView from "./lib/components/RuntimeObserverView.svelte";
  import SidebarBrand from "./lib/components/SidebarBrand.svelte";
  import SubscriptionView from "./lib/components/SubscriptionView.svelte";

  import {
    appendAppEvent as postAppEvent,
    applySubscription as requestSubscriptionApply,
    fetchControlSnapshot,
    fetchLocalNetworkSnapshot,
    refreshSubscription as requestSubscriptionRefresh,
  } from "./lib/api/client";
  import type {
    ControlSnapshotResponse,
    RuntimePaths,
    SingboxBootstrapReport,
    SingboxRuntimeStatus,
  } from "./lib/api/generated";
  import type {
    LocalNetworkSnapshot,
    SubscriptionDefinitionSnapshot,
    SubscriptionRuntimeSnapshot,
  } from "./lib/types";

  type RuntimeEntry = [string, string];

  let runtimePaths: RuntimePaths | null = null;
  let bootstrap: SingboxBootstrapReport | null = null;
  let runtimeStatus: SingboxRuntimeStatus | null = null;
  let error: string | null = null;
  let runtimeItems: RuntimeEntry[] = [];
  let bootstrapItems: RuntimeEntry[] = [];
  let statusItems: RuntimeEntry[] = [];
  let appLog = "";
  let singboxLog = "";
  let sessionRaw = "";
  let runtimeMetadata = "";
  let localNetwork: LocalNetworkSnapshot | null = null;
  let subscription: SubscriptionDefinitionSnapshot | null = null;
  let subscriptionRuntime: SubscriptionRuntimeSnapshot | null = null;
  let activeTab: "overview" | "network" | "subscription" | "observer" = "overview";
  let subscriptionRefreshPending = false;
  let subscriptionApplyPending = false;
  let subscriptionRefreshError: string | null = null;

  function applySnapshot(snapshot: ControlSnapshotResponse) {
    runtimePaths = snapshot.runtime as RuntimePaths;
    bootstrap = snapshot.bootstrap as SingboxBootstrapReport;
    runtimeStatus = snapshot.status as SingboxRuntimeStatus;
    subscription = snapshot.subscription as SubscriptionDefinitionSnapshot;
    subscriptionRuntime = snapshot.subscription_runtime as SubscriptionRuntimeSnapshot;
    appLog = snapshot.app_log;
    singboxLog = snapshot.singbox_log;
    sessionRaw = snapshot.session_raw;
    runtimeMetadata = snapshot.runtime_metadata;
  }

  async function logAppEvent(message: string) {
    try {
      await postAppEvent(message);
    } catch {
      // swallow logging failures during early bootstrap
    }
  }

  async function refreshAll() {
    try {
      const snapshot = await fetchControlSnapshot();
      localNetwork = await fetchLocalNetworkSnapshot();
      applySnapshot(snapshot);
      error = null;
    } catch (err) {
      error = String(err);
      await logAppEvent(`ui refresh failed: ${error}`);
    }
  }

  async function refreshSubscriptionNow() {
    subscriptionRefreshPending = true;
    subscriptionRefreshError = null;
    try {
      subscriptionRuntime = await requestSubscriptionRefresh();
      await refreshAll();
    } catch (err) {
      subscriptionRefreshError = String(err);
      await logAppEvent(`subscription refresh failed: ${subscriptionRefreshError}`);
    } finally {
      subscriptionRefreshPending = false;
    }
  }

  async function applySubscriptionNow() {
    subscriptionApplyPending = true;
    subscriptionRefreshError = null;
    try {
      const result = await requestSubscriptionApply();
      subscriptionRuntime = result.subscription_runtime;
      runtimeStatus = result.status;
      await refreshAll();
    } catch (err) {
      subscriptionRefreshError = String(err);
      await logAppEvent(`subscription apply failed: ${subscriptionRefreshError}`);
    } finally {
      subscriptionApplyPending = false;
    }
  }

  onMount(() => {
    const intervalId = window.setInterval(() => {
      void refreshAll();
    }, 5000);

    void refreshAll();

    return () => {
      window.clearInterval(intervalId);
    };
  });

  $: runtimeItems = runtimePaths
    ? [
        ["mode", runtimePaths.mode],
        ["root", runtimePaths.root],
        ["bin", runtimePaths.binDir],
        ["config", runtimePaths.configDir],
        ["logs", runtimePaths.logsDir],
        ["state", runtimePaths.stateDir],
        ["secrets", runtimePaths.secretsDir],
        ["subscriptions", runtimePaths.subscriptionsDir],
      ] satisfies RuntimeEntry[]
    : [];

  $: bootstrapItems = bootstrap
    ? [
        ["binary", bootstrap.binaryPath],
        ["log", bootstrap.logPath],
        ["pid", bootstrap.pidPath],
        ["session", bootstrap.sessionPath],
        ["process status", bootstrap.processStatus],
        ["version", bootstrap.version ?? "unavailable"],
      ] satisfies RuntimeEntry[]
    : [];

  $: statusItems = runtimeStatus
    ? [
        ["lifecycle", runtimeStatus.lifecycle],
        ["mode", runtimeStatus.mode],
        ["process status", runtimeStatus.processStatus],
        ["pid", runtimeStatus.pid ? String(runtimeStatus.pid) : "none"],
        ["config", runtimeStatus.configPath],
        ["log", runtimeStatus.logPath],
      ] satisfies RuntimeEntry[]
    : [];

</script>

<div class="shell">
  <div class="shell-layout">
    <SidebarBrand activeTab={activeTab} onSelect={(tab) => (activeTab = tab)} />

    <main class="page-main">
      <div class="page-stack">
        {#if activeTab === "overview"}
          <HeroHeader />

          {#if error}
            <section class="alert-error">
              {error}
            </section>
          {/if}

          <section class="grid gap-6 xl:grid-cols-2">
            <Panel title="Runtime Paths" badge="readable" badgeClass="badge-readable">
              <KeyValueList items={runtimeItems} />
            </Panel>

            <Panel title="Process Status" badge="live" badgeClass="badge-live">
              <KeyValueList items={statusItems} />
            </Panel>

            <Panel title="Lifecycle Bootstrap" badge="writable" badgeClass="badge-writable">
              <KeyValueList items={bootstrapItems} />
            </Panel>
          </section>

          <Panel title="Bootstrap Checks" badge="raw state dump" badgeClass="badge-subtle">
            <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
              {#each bootstrap?.checks ?? [] as check}
                <article class={check.ok ? "check-card-pass" : "check-card-fail"}>
                  <div class="check-header">
                    <strong class="check-name">{check.name}</strong>
                    <span class={check.ok ? "check-badge-pass" : "check-badge-fail"}>
                      {check.ok ? "OK" : "FAIL"}
                    </span>
                  </div>
                  <code class="check-detail">{check.detail}</code>
                </article>
              {/each}
            </div>
          </Panel>

          <section class="grid gap-6 xl:grid-cols-2">
            <Panel title="App Log" badge="api">
              <LogBlock content={appLog} />
            </Panel>

            <Panel title="Sing-box Log" badge="api">
              <LogBlock content={singboxLog} />
            </Panel>
          </section>

          <section class="grid gap-6 xl:grid-cols-2">
            <Panel title="Session Raw" badge="state/session.json">
              <LogBlock content={sessionRaw} />
            </Panel>

            <Panel title="Runtime Metadata" badge="metadata/runtime.json">
              <LogBlock content={runtimeMetadata} />
            </Panel>
          </section>
        {:else if activeTab === "network"}
          <LocalNetworkView network={localNetwork} />
        {:else if activeTab === "subscription"}
          <SubscriptionView {subscription} />
        {:else}
          <RuntimeObserverView
            runtimeStatus={runtimeStatus}
            subscriptionRuntime={subscriptionRuntime}
            refreshPending={subscriptionRefreshPending}
            applyPending={subscriptionApplyPending}
            refreshError={subscriptionRefreshError}
            onRefresh={refreshSubscriptionNow}
            onApply={applySubscriptionNow}
          />
        {/if}
      </div>
    </main>
  </div>
</div>
