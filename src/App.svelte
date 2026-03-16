<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import type { RuntimePaths, SingboxBootstrapReport } from "./lib/types";

  type RuntimeEntry = [string, string];

  let runtimePaths: RuntimePaths | null = null;
  let bootstrap: SingboxBootstrapReport | null = null;
  let error: string | null = null;
  let runtimeItems: RuntimeEntry[] = [];
  let bootstrapItems: RuntimeEntry[] = [];

  onMount(async () => {
    try {
      const [paths, report] = await Promise.all([
        invoke<RuntimePaths>("get_runtime_paths"),
        invoke<SingboxBootstrapReport>("bootstrap_singbox"),
      ]);

      runtimePaths = paths;
      bootstrap = report;
      error = null;
    } catch (err) {
      error = String(err);
    }
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
</script>

<div class="min-h-screen bg-transparent text-slate-100">
  <div class="flex min-h-screen">
    <aside class="w-64 border-r border-white/8 bg-slate-950/55 px-4 py-5 backdrop-blur-xl">
      <div class="mb-8 flex items-center gap-3 rounded-2xl bg-white/6 p-3 ring-1 ring-white/10">
        <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-linear-to-br from-sky-500 to-blue-700 font-semibold text-white shadow-lg shadow-sky-900/40">
          SB
        </div>
        <div>
          <div class="text-sm font-semibold tracking-wide text-white">singbox-tauri</div>
          <div class="text-xs text-slate-400">local control plane</div>
        </div>
      </div>

      <nav class="space-y-2">
        <div class="rounded-2xl bg-sky-500/14 px-4 py-3 text-sm font-medium text-sky-200 ring-1 ring-sky-400/20">
          singbox 控制面板
        </div>
      </nav>
    </aside>

    <main class="flex-1 px-6 py-6 lg:px-8">
      <div class="mx-auto max-w-7xl space-y-6">
        <header class="rounded-3xl border border-white/8 bg-slate-900/45 px-6 py-6 shadow-2xl shadow-slate-950/30 backdrop-blur-xl">
          <p class="text-xs font-semibold uppercase tracking-[0.22em] text-sky-300/80">dashboard bootstrap</p>
          <h1 class="mt-2 text-3xl font-semibold tracking-tight text-white">singbox 控制面板</h1>
          <p class="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
            先把当前可读、可写的运行态信息全部摊平。后续只在需要的时候继续细分，不做多余防御。
          </p>
        </header>

        {#if error}
          <section class="rounded-2xl border border-rose-400/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-100 shadow-lg shadow-rose-950/20">
            {error}
          </section>
        {/if}

        <section class="grid gap-6 xl:grid-cols-2">
          <div class="rounded-3xl border border-white/8 bg-slate-900/45 p-5 shadow-2xl shadow-slate-950/30 backdrop-blur-xl">
            <div class="mb-4 flex items-center justify-between">
              <h2 class="text-lg font-semibold text-white">Runtime Paths</h2>
              <span class="rounded-full bg-slate-800/80 px-3 py-1 text-xs text-slate-300">readable</span>
            </div>
            <div class="space-y-3">
              {#each runtimeItems as [label, value]}
                <div class="rounded-2xl border border-white/6 bg-slate-950/45 px-4 py-3">
                  <div class="mb-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">{label}</div>
                  <code class="block break-all text-sm text-slate-100">{value}</code>
                </div>
              {/each}
            </div>
          </div>

          <div class="rounded-3xl border border-white/8 bg-slate-900/45 p-5 shadow-2xl shadow-slate-950/30 backdrop-blur-xl">
            <div class="mb-4 flex items-center justify-between">
              <h2 class="text-lg font-semibold text-white">Lifecycle Bootstrap</h2>
              <span class="rounded-full bg-emerald-500/12 px-3 py-1 text-xs text-emerald-200">writable</span>
            </div>
            <div class="space-y-3">
              {#each bootstrapItems as [label, value]}
                <div class="rounded-2xl border border-white/6 bg-slate-950/45 px-4 py-3">
                  <div class="mb-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">{label}</div>
                  <code class="block break-all text-sm text-slate-100">{value}</code>
                </div>
              {/each}
            </div>
          </div>
        </section>

        <section class="rounded-3xl border border-white/8 bg-slate-900/45 p-5 shadow-2xl shadow-slate-950/30 backdrop-blur-xl">
          <div class="mb-4 flex items-center justify-between">
            <h2 class="text-lg font-semibold text-white">Bootstrap Checks</h2>
            <span class="rounded-full bg-white/8 px-3 py-1 text-xs text-slate-300">raw state dump</span>
          </div>

          <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
            {#each bootstrap?.checks ?? [] as check}
              <article class={`rounded-2xl border px-4 py-4 ${check.ok ? "border-emerald-400/18 bg-emerald-500/8" : "border-rose-400/20 bg-rose-500/8"}`}>
                <div class="mb-3 flex items-center justify-between gap-3">
                  <strong class="text-sm font-semibold text-white">{check.name}</strong>
                  <span class={`rounded-full px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] ${check.ok ? "bg-emerald-400/14 text-emerald-200" : "bg-rose-400/12 text-rose-200"}`}>
                    {check.ok ? "OK" : "FAIL"}
                  </span>
                </div>
                <code class="block break-all text-sm leading-6 text-slate-200">{check.detail}</code>
              </article>
            {/each}
          </div>
        </section>
      </div>
    </main>
  </div>
</div>
