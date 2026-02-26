<script lang="ts">
  import "../app.css"
  import Sidebar from "$lib/components/Sidebar.svelte"
  import SettingsView from "$lib/components/SettingsView.svelte"
  import type { Snippet } from "svelte"

  interface Props {
    children: Snippet
  }

  let { children }: Props = $props()

  let activeTab = $state<"general" | "settings">("general")

  function handleTabChange(tab: "general" | "settings") {
    activeTab = tab
  }
</script>

<div class="grid grid-cols-[auto_1fr] h-screen">
  <Sidebar {activeTab} onTabChange={handleTabChange} />
  <main class="overflow-y-auto">
    {#if activeTab === "general"}
      {@render children()}
    {:else}
      <SettingsView />
    {/if}
  </main>
</div>
