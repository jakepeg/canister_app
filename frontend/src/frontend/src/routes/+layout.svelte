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

<div class="flex flex-col h-screen overflow-hidden gap-y-2">
  <ModeWatcher />
  <Navbar />
  <div class="flex flex-1 overflow-hidden">
    {#if $authStore.state === "authenticated"}
      <Sidebar.Provider>
        {#if showSidebar}
          <AppSidebar />
        {/if}

        {#if showSidebar}
          <Sidebar.Inset class="flex-1 flex overflow-auto">
            <main class="flex-1">
              <Sidebar.Trigger class="sticky top-4 left-4 z-10" />
              <div class="max-w-5xl mx-auto px-4 ml-10">
                {@render children?.()}
              </div>
            </main>
          </Sidebar.Inset>
        {:else}
          <main class="flex-1 overflow-auto">
            <div class="max-w-5xl mx-auto px-4">
              {@render children?.()}
            </div>
          </main>
        {/if}
      </Sidebar.Provider>
    {:else}
      <main class="flex-1 overflow-y-auto">
        <div class="max-w-5xl mx-auto px-4">
          {@render children?.()}
        </div>
      </main>
    {/if}
  </div>
</div>

{#if $authStore.state === "authenticated" && $userStore.state === "unregistered"}
  <RegistrationModal
    isOpen={$userStore.state === "unregistered"}
    authenticatedStore={$authStore}
  />
{/if}
