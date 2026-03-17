<script lang="ts">
  export let title: string;
  export let badge: string | null = null;
  export let badgeClass = "badge-muted";
  export let panelClass = "panel";
</script>

<section class={panelClass}>
  <div class="panel-header">
    <h2 class="panel-title">{title}</h2>
    {#if badge}
      <span class={badgeClass}>{badge}</span>
    {/if}
  </div>
  <slot />
</section>
