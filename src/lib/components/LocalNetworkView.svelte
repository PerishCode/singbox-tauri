<script lang="ts">
  import type { LocalNetworkSnapshot, NetworkInterfaceSummary, NetworkProxyStatus } from "../types";
  import JsonPreview from "./JsonPreview.svelte";
  import Panel from "./Panel.svelte";

  export let network: LocalNetworkSnapshot | null = null;

  $: activeInterfaces = network
    ? network.interfaces.filter((item: NetworkInterfaceSummary) => item.isActive)
    : [];

  $: enabledProxies = network
    ? network.proxies.filter((item: NetworkProxyStatus) => item.enabled)
    : [];

  $: readinessView = network
    ? {
        readiness: network.readiness,
        headline: network.headline,
        reasons: network.reasons,
      }
    : null;

  $: networkStateView = network
    ? {
        readiness: network.readiness,
        defaultInterface: network.defaultRoute.interface ?? "unknown",
        defaultGateway: network.defaultRoute.gateway ?? "unknown",
        systemProxyEnabled: network.systemProxyEnabled,
        utunInterfaces: network.utunInterfaces,
      }
    : null;

  $: proxyDnsView = network
    ? {
        enabledProxies,
        dnsResolvers: network.dnsResolvers,
      }
    : null;

  $: conflictView = network
    ? {
        relatedProcesses: network.relatedProcesses,
        portBindings: network.portBindings,
        conflicts: network.conflicts,
      }
    : null;

  $: diagnosticsView = network
    ? {
        defaultRoute: network.diagnostics.defaultRouteRaw,
        proxy: network.diagnostics.proxyRaw,
        dns: network.diagnostics.dnsRaw,
      }
    : null;
</script>

<div class="space-y-6">
  <header class="panel space-y-3">
    <div class="badge preset-tonal-primary inline-flex">local network</div>
    <h1 class="text-3xl font-semibold tracking-tight text-white">本地网络状态</h1>
    <p class="max-w-3xl text-sm leading-6 text-slate-300">
      先观察当前机器的路由、代理、utun 和相关进程占用情况。
    </p>
  </header>

  {#if network}
    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Readiness" badge={network.readiness}>
        <JsonPreview value={readinessView} />
      </Panel>

      <Panel title="Network State" badge="observed" badgeClass="badge preset-tonal-primary">
        <JsonPreview value={networkStateView} />
      </Panel>
    </section>

    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Interfaces" badge={String(activeInterfaces.length)} badgeClass="badge preset-tonal-primary">
        <JsonPreview value={activeInterfaces} emptyText="No active interfaces parsed from current snapshot." />
      </Panel>

      <Panel title="Proxy and DNS" badge="structured">
        <JsonPreview value={proxyDnsView} />
      </Panel>
    </section>

    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Conflict Signals" badge={network.conflicts.length ? String(network.conflicts.length) : "clear"} badgeClass={network.conflicts.length ? "badge preset-tonal-warning" : "badge preset-tonal-surface"}>
        <JsonPreview value={conflictView} />
      </Panel>

      <Panel title="Raw Diagnostics" badge="macOS">
        <JsonPreview value={diagnosticsView} />
      </Panel>
    </section>
  {:else}
    <section class="alert-error">Local network snapshot is unavailable.</section>
  {/if}
</div>
