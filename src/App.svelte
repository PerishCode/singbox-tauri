<script lang="ts">
  import { AppBar, Navigation } from "@skeletonlabs/skeleton-svelte";
  import { Router, route, type RouteConfig } from "@mateothegreat/svelte5-router";
  import { onMount } from "svelte";
  import OverviewPage from "./pages/OverviewPage.svelte";
  import NetworkPage from "./pages/NetworkPage.svelte";
  import SubscriptionPage from "./pages/SubscriptionPage.svelte";
  import ObserverPage from "./pages/ObserverPage.svelte";

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

  let runtimePaths: RuntimePaths | null = null;
  let bootstrap: SingboxBootstrapReport | null = null;
  let runtimeStatus: SingboxRuntimeStatus | null = null;
  let error: string | null = null;
  let appLog = "";
  let singboxLog = "";
  let sessionRaw = "";
  let runtimeMetadata = "";
  let localNetwork: LocalNetworkSnapshot | null = null;
  let subscription: SubscriptionDefinitionSnapshot | null = null;
  let subscriptionRuntime: SubscriptionRuntimeSnapshot | null = null;
  let subscriptionRefreshPending = false;
  let subscriptionApplyPending = false;
  let subscriptionRefreshError: string | null = null;
  let routes: RouteConfig[] = [];

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

  $: routes = [
    {
      path: "/",
      component: OverviewPage,
      props: {
        error,
        runtimePaths,
        runtimeStatus,
        bootstrap,
        checks: bootstrap?.checks ?? [],
        appLog,
        singboxLog,
        sessionRaw,
        runtimeMetadata,
      },
    },
    {
      path: "/network",
      component: NetworkPage,
      props: { localNetwork },
    },
    {
      path: "/subscription",
      component: SubscriptionPage,
      props: { subscription },
    },
    {
      path: "/observer",
      component: ObserverPage,
      props: {
        runtimeStatus,
        subscriptionRuntime,
        refreshPending: subscriptionRefreshPending,
        applyPending: subscriptionApplyPending,
        refreshError: subscriptionRefreshError,
        onRefresh: refreshSubscriptionNow,
        onApply: applySubscriptionNow,
      },
    },
  ];
</script>

<div class="shell" data-theme="cerberus">
  <main class="page-main">
    <div class="page-stack">
      <AppBar class="app-shell-bar">
        <AppBar.Toolbar class="app-shell-toolbar">
          <AppBar.Lead class="app-shell-brand">
            <div class="brand-mark">SB</div>
            <div>
              <div class="text-sm font-semibold tracking-wide">singbox-tauri</div>
              <div class="text-xs opacity-70">local control plane</div>
            </div>
          </AppBar.Lead>
          <AppBar.Trail>
            <span class="badge preset-outlined-primary-400-600">Skeleton + router</span>
          </AppBar.Trail>
        </AppBar.Toolbar>
      </AppBar>

      <Navigation layout="bar" class="card preset-filled-surface-900-100 border border-surface-800/70 p-2">
        <Navigation.Menu class="flex flex-wrap gap-2">
          <a href="/" use:route={{ active: { class: "route-link-active", absolute: true }, default: { class: "route-link" } }}>Overview</a>
          <a href="/network" use:route={{ active: { class: "route-link-active", absolute: true }, default: { class: "route-link" } }}>Local Network</a>
          <a href="/subscription" use:route={{ active: { class: "route-link-active", absolute: true }, default: { class: "route-link" } }}>Subscription</a>
          <a href="/observer" use:route={{ active: { class: "route-link-active", absolute: true }, default: { class: "route-link" } }}>Runtime Observer</a>
        </Navigation.Menu>
      </Navigation>

      <div class="route-view">
        <Router {routes} />
      </div>
    </div>
  </main>
</div>
