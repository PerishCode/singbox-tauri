<script lang="ts">
  import type { SidebarTab } from "./sidebar";

  export let activeTab: SidebarTab = "overview";
  export let onSelect: (tab: SidebarTab) => void;
</script>

<aside class="sidebar">
  <div class="brand-card">
    <div class="brand-mark">
      SB
    </div>
    <div>
      <div class="text-sm font-semibold tracking-wide text-white">singbox-tauri</div>
      <div class="text-xs text-slate-400">local control plane</div>
    </div>
  </div>

  <nav class="space-y-2">
    <button class={activeTab === "overview" ? "nav-item-active" : "nav-item"} on:click={() => onSelect("overview")}>
      Overview
    </button>
    <button class={activeTab === "network" ? "nav-item-active" : "nav-item"} on:click={() => onSelect("network")}>
      Local Network
    </button>
    <button class={activeTab === "subscription" ? "nav-item-active" : "nav-item"} on:click={() => onSelect("subscription")}>
      Subscription
    </button>
  </nav>
</aside>
