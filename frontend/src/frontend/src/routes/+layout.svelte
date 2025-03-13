<script lang="ts">
  import Navbar from "$lib/components/Navbar.svelte";
  import AppSidebar from "$lib/components/Sidebar.svelte";
  import AppSidebarv2 from "$lib/components/AppSidebar.svelte";
  import RegistrationModal from "$lib/components/RegistrationModal.svelte";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { onMount } from "svelte";
  import "../app.css";
  // import Disclaimer from "$lib/components/Disclaimer.svelte";
  import { ModeWatcher } from "mode-watcher";
  import * as Sidebar from "$lib/components/ui/sidebar";

  const title = "Canister – Encrypted document sharing and requesting";
  const description =
    "Effortless document sharing on the Internet Computer. No plugins, no passwords. Canister enables seamless document requests, streamlining interactions for service providers and clients.";
  const image = `https://${
    import.meta.env.VITE_FRONTEND_CANISTER_ID
  }.icp0.io/share.jpg`;
  const url = `https://${
    import.meta.env.VITE_FRONTEND_CANISTER_ID
  }.icp0.io{$page.url.pathname}`;
  const domain = `${import.meta.env.VITE_FRONTEND_CANISTER_ID}.icp0.io`;

  onMount(async () => {
    authService.init();
  });

  const unregistered = $derived(
    $authStore.state === "authenticated" && $userStore.state === "unregistered",
  );

  let { children } = $props();
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />

  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:type" content="website" />
  <meta name="og:image" content={image} />
  <meta property="og:url" content={url} />

  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:site" content="@dfinity" />
  <meta name="twitter:title" content={title} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={image} />
  <meta property="twitter:url" content={url} />
  <meta property="twitter:domain" content={domain} />
</svelte:head>

<div class="flex flex-col h-screen overflow-hidden gap-y-2">
  <ModeWatcher />
  <Navbar />
  <div class="flex flex-1 overflow-hidden">
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
  </div>
</div>

{#if $authStore.state === "authenticated" && $userStore.state === "unregistered"}
  <RegistrationModal isOpen={unregistered} authenticatedStore={$authStore} />
{/if}
