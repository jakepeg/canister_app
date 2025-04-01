<script lang="ts">
  import Navbar from "$lib/components/Navbar.svelte";
  import AppSidebar from "$lib/components/Sidebar.svelte";
  import AppSidebarv2 from "$lib/components/AppSidebar.svelte";
  import RegistrationModal from "$lib/components/RegistrationModal.svelte";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { onMount } from "svelte";
  import { page } from '$app/stores'; // Import page store
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import * as Sidebar from "$lib/components/ui/sidebar";

  onMount(async () => {
    authService.init();

    // Apply dark mode for non-authenticated users
    if ($authStore.state !== "authenticated") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  });

  let { children } = $props();
</script>

{#if $authStore.state === "authenticated"}
  <!-- Authenticated layout -->
  <div class="flex flex-col h-screen overflow-hidden gap-y-2">
    <ModeWatcher />
    <Navbar />
    <div class="flex flex-1 overflow-hidden">
      {#if $page.route.id?.startsWith('/canister/')}
        <!-- Layout WITH Sidebar (for canister-specific pages) -->
        <Sidebar.Provider>
          <AppSidebar />
          <Sidebar.Inset class="flex-1 flex overflow-auto">
            <main class="flex-1">
              <Sidebar.Trigger class="sticky top-4 left-4" />
              <div class="max-w-5xl mx-auto px-4 ml-10">
                {@render children?.()}
              </div>
            </main>
          </Sidebar.Inset>
        </Sidebar.Provider>
      {:else}
        <!-- Layout WITHOUT Sidebar (for main page /) -->
        <main class="flex-1 overflow-auto">
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
{:else}
  <!-- No layout applied for non-authenticated users -->
  <div class="flex flex-col h-screen overflow-hidden gap-y-2">
    <ModeWatcher />
    <Navbar />
    <div class="flex flex-1 overflow-hidden">
      <main class="flex-1">
        <div class="max-w-5xl mx-auto px-4">
          {@render children?.()}
        </div>
      </main>
    </div>
  </div>
{/if}
