<script lang="ts">
  import type { LocalNetworkSnapshot, NetworkInterfaceSummary, NetworkProxyStatus } from "../types";
  import LogBlock from "./LogBlock.svelte";
  import Panel from "./Panel.svelte";

  export let network: LocalNetworkSnapshot | null = null;

  $: summaryItems = network
    ? [
        ["readiness", network.readiness],
        ["default interface", network.defaultRoute.interface ?? "unknown"],
        ["default gateway", network.defaultRoute.gateway ?? "unknown"],
        ["system proxy", network.systemProxyEnabled ? "enabled" : "disabled"],
        ["utun interfaces", network.utunInterfaces.length ? network.utunInterfaces.join(", ") : "none"],
      ]
    : [];

  $: activeInterfaces = network
    ? network.interfaces.filter((item: NetworkInterfaceSummary) => item.isActive)
    : [];
  $: enabledProxies = network
    ? network.proxies.filter((item: NetworkProxyStatus) => item.enabled)
    : [];
</script>

<div class="space-y-6">
  <section class="panel-hero">
    <p class="text-xs font-semibold uppercase tracking-[0.22em] text-sky-300/80">local network</p>
    <h1 class="mt-2 text-3xl font-semibold tracking-tight text-white">本地网络状态</h1>
    <p class="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
      先观察当前机器的路由、代理、utun 和相关进程占用情况，为后续 sing-box TUN 灰度接入提供依据。
    </p>
  </section>

  {#if network}
    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Readiness" badge={network.readiness} badgeClass="badge-subtle">
        <div class="space-y-4">
          <div class="value-card">
            <div class="value-label">headline</div>
            <div class="text-sm leading-6 text-slate-100">{network.headline}</div>
          </div>
          <div class="value-card">
            <div class="value-label">reasons</div>
            <ul class="space-y-2 text-sm leading-6 text-slate-200">
              {#each network.reasons as reason}
                <li>{reason}</li>
              {/each}
            </ul>
          </div>
        </div>
      </Panel>

      <Panel title="Network State" badge="observed" badgeClass="badge-readable">
        <div class="space-y-3">
          {#each summaryItems as [label, value]}
            <div class="value-card">
              <div class="value-label">{label}</div>
              <code class="value-code">{value}</code>
            </div>
          {/each}
        </div>
      </Panel>
    </section>

    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Interfaces" badge={String(activeInterfaces.length)} badgeClass="badge-readable">
        <div class="space-y-3">
          {#if activeInterfaces.length}
            {#each activeInterfaces as item}
              <div class="value-card">
                <div class="value-label">{item.name}</div>
                <code class="value-code">{item.kind} / {item.addresses.length ? item.addresses.join(", ") : "no address"}</code>
              </div>
            {/each}
          {:else}
            <div class="value-card">
              <div class="text-sm text-slate-300">No active interfaces parsed from current snapshot.</div>
            </div>
          {/if}
        </div>
      </Panel>

      <Panel title="Proxy and DNS" badge="structured" badgeClass="badge-subtle">
        <div class="space-y-4">
          <div class="value-card">
            <div class="value-label">enabled proxies</div>
            {#if enabledProxies.length}
              <ul class="space-y-2 text-sm leading-6 text-slate-200">
                {#each enabledProxies as proxy}
                  <li>{proxy.kind} - {proxy.host ?? "host unknown"}{proxy.port ? `:${proxy.port}` : ""}</li>
                {/each}
              </ul>
            {:else}
              <div class="text-sm text-slate-300">No enabled HTTP/HTTPS/SOCKS system proxies parsed.</div>
            {/if}
          </div>
          <div class="value-card">
            <div class="value-label">dns resolvers</div>
            {#if network.dnsResolvers.length}
              <ul class="space-y-2 text-sm leading-6 text-slate-200">
                {#each network.dnsResolvers as resolver}
                  <li>{resolver.scope}: {resolver.resolvers.join(", ")}</li>
                {/each}
              </ul>
            {:else}
              <div class="text-sm text-slate-300">No DNS resolvers parsed from current snapshot.</div>
            {/if}
          </div>
        </div>
      </Panel>
    </section>

    <section class="grid gap-6 xl:grid-cols-2">
      <Panel title="Conflict Signals" badge={network.conflicts.length ? String(network.conflicts.length) : "clear"} badgeClass="badge-live">
        <div class="space-y-4 text-sm text-slate-200">
          <div class="value-card">
            <div class="value-label">processes</div>
            {#if network.relatedProcesses.length}
              <ul class="space-y-2 leading-6">
                {#each network.relatedProcesses as process}
                  <li><code class="value-code">{process.label} (pid {process.pid}) - {process.command}</code></li>
                {/each}
              </ul>
            {:else}
              <div class="text-sm text-slate-300">No watched proxy processes detected.</div>
            {/if}
          </div>

          <div class="value-card">
            <div class="value-label">ports</div>
            {#if network.portBindings.length}
              <ul class="space-y-2 leading-6">
                {#each network.portBindings as binding}
                  <li><code class="value-code">:{binding.port} {binding.process}</code></li>
                {/each}
              </ul>
            {:else}
              <div class="text-sm text-slate-300">No watched proxy ports are currently occupied.</div>
            {/if}
          </div>

          <div class="value-card">
            <div class="value-label">conflicts</div>
            {#if network.conflicts.length}
              <ul class="space-y-3 leading-6">
                {#each network.conflicts as conflict}
                  <li>
                    <div class="text-sm font-semibold text-white">{conflict.level} - {conflict.message}</div>
                    <div class="text-xs text-slate-400">{conflict.code}</div>
                  </li>
                {/each}
              </ul>
            {:else}
              <div class="text-sm text-slate-300">No explicit conflicts were raised in the first pass.</div>
            {/if}
          </div>
        </div>
      </Panel>

      <Panel title="Raw Diagnostics" badge="macOS" badgeClass="badge-muted">
        <div class="space-y-4">
          <div>
            <div class="value-label">route -n get default</div>
            <LogBlock content={network.diagnostics.defaultRouteRaw} />
          </div>
          <div>
            <div class="value-label">scutil --proxy</div>
            <LogBlock content={network.diagnostics.proxyRaw} />
          </div>
          <div>
            <div class="value-label">scutil --dns</div>
            <LogBlock content={network.diagnostics.dnsRaw} />
          </div>
        </div>
      </Panel>
    </section>
  {:else}
    <section class="alert-error">
      Local network snapshot is unavailable.
    </section>
  {/if}
</div>
