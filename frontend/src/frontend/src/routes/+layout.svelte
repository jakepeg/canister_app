<script lang="ts">
  import Navbar from "$lib/components/Navbar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import RegistrationModal from "$lib/components/RegistrationModal.svelte";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { onMount } from "svelte";
  import "../app.css";
  // import Disclaimer from "$lib/components/Disclaimer.svelte";
  import { ModeWatcher } from "mode-watcher";

  const title = "DocuTrack – Encrypted document sharing and requesting";
  const description =
    "Effortless document sharing on the Internet Computer. No plugins, no passwords. DocuTrack enables seamless document requests, streamlining interactions for service providers and clients.";
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

  $: unregistered =
    $authStore.state === "authenticated" && $userStore.state === "unregistered";
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

<div class="flex flex-col min-h-screen">
  <ModeWatcher />
  <Navbar />
  <div class="flex flex-1">
    <Sidebar />
    <main class="bg-muted/40 flex-1 p-4">
      <!-- <Disclaimer /> -->
      <div class="max-w-5xl px-4 mx-auto pt-6">
        <slot />
      </div>
    </main>
  </div>
</div>

{#if $authStore.state === "authenticated" && $userStore.state === "unregistered"}
  <RegistrationModal isOpen={unregistered} authenticatedStore={$authStore} />
{/if}
