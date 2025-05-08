<script lang="ts">
  import Navbar from "$lib/components/Navbar.svelte";
  import AppSidebar from "$lib/components/Sidebar.svelte";
  // import AppSidebarv2 from "$lib/components/AppSidebar.svelte"; // Assuming AppSidebar is the one you use
  import RegistrationModal from "$lib/components/RegistrationModal.svelte";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import "../app.css";
  import { ModeWatcher, resetMode, setMode } from "mode-watcher";
  import * as Sidebar from "$lib/components/ui/sidebar"; // Keep if AppSidebar relies on it

  // Move $effect inside onMount or ensure it runs correctly in Svelte 5 context
  // For simplicity, let's stick to the original onMount logic for mode setting
  onMount(async () => {
    authService.init();
    // Apply dark mode for non-authenticated users
    // if ($authStore.state !== "authenticated") {
    //   setMode("dark");
    // } else {
    //   resetMode();
    // }
  });

  $effect(() => {
    if ($authStore.state !== "authenticated") {
      setMode("dark");
    } else {
      resetMode();
    }
  });

  let { children } = $props();

  // Determine if sidebar should be shown based on the route
  // Replace $: with let ... = $derived()
  let showSidebar = $derived(
    $authStore.state === "authenticated" &&
      $page.route.id?.startsWith("/canister/"),
  );
</script>

<div class="flex flex-col h-screen overflow-hidden">
  <ModeWatcher />
  <Navbar />

  <div class="flex flex-1 overflow-hidden">
    <!-- Conditionally render the Sidebar component itself -->
    {#if showSidebar}
      <Sidebar.Provider>
        <AppSidebar />
      </Sidebar.Provider>
    {/if}

    <!-- Main content area - ALWAYS rendered -->
    <main
      class="flex-1 overflow-y-auto p-4 transition-all duration-300 ease-in-out"
      class:md:ml-64={showSidebar}
      class:ml-0={!showSidebar}
    >
      {#if showSidebar}
        <!-- Position trigger within main, maybe absolutely or relatively if needed -->
        <Sidebar.Trigger
          class="fixed top-18 left-4 z-10 md:hidden p-2 rounded bg-background/80 backdrop-blur-sm border"
        />
      {/if}

      <!-- Container for page content -->
      <div class="max-w-5xl mx-auto">
        {@render children?.()}
      </div>
    </main>
  </div>
</div>
<!-- Registration modal remains outside -->
{#if $authStore.state === "authenticated" && $userStore.state === "unregistered"}
  <RegistrationModal
    isOpen={$userStore.state === "unregistered"}
    authenticatedStore={$authStore}
  />
{/if}
