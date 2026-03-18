<script lang="ts">
  import JsonPreview from "../lib/components/JsonPreview.svelte";
  import Panel from "../lib/components/Panel.svelte";
  import type { RuntimePaths, SingboxBootstrapReport, SingboxRuntimeStatus } from "../lib/api/generated";

  export let error: string | null = null;
  export let runtimePaths: RuntimePaths | null = null;
  export let runtimeStatus: SingboxRuntimeStatus | null = null;
  export let bootstrap: SingboxBootstrapReport | null = null;
  export let checks: Array<{ name: string; ok: boolean; detail: string }> = [];
  export let appLog = "";
  export let singboxLog = "";
  export let sessionRaw = "";
  export let runtimeMetadata = "";
</script>

<div class="space-y-6">
  <header class="panel space-y-3">
    <div class="badge preset-tonal-primary inline-flex">dashboard bootstrap</div>
    <h1 class="text-3xl font-semibold tracking-tight text-white">singbox 控制面板</h1>
    <p class="max-w-3xl text-sm leading-6 text-slate-300">
      应用启动后默认尝试准备并拉起 sing-box。控制面当前保持只读，优先展示运行态、网络环境和失败证据。
    </p>
  </header>

  {#if error}
    <section class="alert-error">{error}</section>
  {/if}

  <section class="grid gap-6 xl:grid-cols-2">
    <Panel title="Runtime Paths" badge="readable" badgeClass="badge preset-tonal-primary">
      <JsonPreview value={runtimePaths} />
    </Panel>

    <Panel title="Process Status" badge="live" badgeClass="badge preset-tonal-warning">
      <JsonPreview value={runtimeStatus} />
    </Panel>

    <Panel title="Lifecycle Bootstrap" badge="writable" badgeClass="badge preset-tonal-success">
      <JsonPreview value={bootstrap} />
    </Panel>
  </section>

  <Panel title="Bootstrap Checks" badge="raw state dump">
    <JsonPreview value={checks} />
  </Panel>

  <section class="grid gap-6 xl:grid-cols-2">
    <Panel title="App Log" badge="api">
      <JsonPreview value={appLog} />
    </Panel>

    <Panel title="Sing-box Log" badge="api">
      <JsonPreview value={singboxLog} />
    </Panel>
  </section>

  <section class="grid gap-6 xl:grid-cols-2">
    <Panel title="Session Raw" badge="state/session.json">
      <JsonPreview value={sessionRaw} />
    </Panel>

    <Panel title="Runtime Metadata" badge="metadata/runtime.json">
      <JsonPreview value={runtimeMetadata} />
    </Panel>
  </section>
</div>
