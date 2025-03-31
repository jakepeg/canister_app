<script lang="ts">
  import { page } from "$app/stores";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { fade, fly } from "svelte/transition";
  import IconFile from "./icons/IconFile.svelte";
  import LogoIcon from "./icons/LogoIcon.svelte";
  import LogoutIcon from "./icons/LogoutIcon.svelte";
  import RequestsIcon from "./icons/RequestsIcon.svelte";
  import UploadIcon from "./icons/UploadIcon.svelte";
  import { uploadInProgress } from "$lib/services/upload";
  import ModeToggle from "$lib/components/mode-toggle.svelte";
  import { Button } from "$lib/components/ui/button";

  let showMobileMenu = false;

  function logout() {
    if ($uploadInProgress) {
      if (
        !confirm("Uploading is in progress. Are you sure you want to logout?")
      )
        return;
    }

    authService.logout();
  }
</script>

<nav class="bg-sidebar w-full relative z-20">
  <div class="flex h-14 md:h-16 items-center px-4 justify-between">
    <!-- Left side with logo -->
    <div class="flex items-center gap-2">
      <a href="/" class="shrink-0">
        <img src="/logo.svg" alt="" class="hidden lg:block h-10 w-10" />
        <img src="/mobile-logo1.svg" alt="" class="lg:hidden h-10 w-10" />
      </a>
      CANISTER

      {#if $authStore.state === "authenticated" && $userStore.state === "registered"}
        <div
          class="bg-accent-100/10 p-2 rounded-lg rounded-bl-none text-accent-100 body-1"
        >
          Hi, {$userStore.username}.
        </div>
      {/if}
    </div>

    <!-- Right side with buttons -->
    <div class="flex items-center ml-auto">
      {#if $authStore.state === "authenticated"}
        <ModeToggle />
      {/if}

      {#if $authStore.state === "unauthenticated"}
        <button
          class="gap-4 btn btn-accent"
          on:click={() => authService.login()}
        >
          <LogoIcon />
          Login
        </button>
      {:else if $authStore.state === "authenticated"}
        <button
          class="flex flex-col items-stretch gap-[5px] md:hidden w-5 h-5"
          on:click={() => (showMobileMenu = !showMobileMenu)}
          aria-label="Open menu"
        >
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-transform {showMobileMenu
              ? 'rotate-45 translate-y-[7px]'
              : 'rotate-0'}"
          >
          </span>
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-opacity {showMobileMenu
              ? 'opacity-0'
              : 'opacity-100'}"
          >
          </span>
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-transform {showMobileMenu
              ? '-rotate-45 translate-y-[-7px]'
              : 'rotate-0'}"
          >
          </span>
        </button>

        <div class="hidden md:flex gap-2 lg:gap-8">
          <button on:click={() => logout()} class="btn btn-ghost">
            <LogoutIcon />
          </button>
        </div>
      {/if}
    </div>
  </div>
</nav>

{#if showMobileMenu}
  <div
    class="md:hidden fixed inset-0 bg-black/50"
    transition:fade|global={{ duration: 200 }}
  >
  </div>
  <div
    transition:fly|global={{ duration: 300, x: 1000, opacity: 1 }}
    class="fixed md:hidden inset-0 bg-background-300 z-10 pt-16"
  >
    <div class="p-4 flex flex-col gap-4 h-full">
      <a
        href="/"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/"}
        on:click={() => (showMobileMenu = false)}
      >
        <IconFile />
        Files</a
      >
      <a
        href="/upload"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/upload"}
        on:click={() => (showMobileMenu = false)}
      >
        <UploadIcon />
        Upload</a
      >
      <a
        href="/requests"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/requests"}
        on:click={() => (showMobileMenu = false)}
      >
        <RequestsIcon />
        Requests</a
      >
      <div class="flex-1" ></div>
      <button
        on:click={() => {
          authService.logout();
          showMobileMenu = false;
        }}
        class="btn btn-ghost justify-start"
      >
        <LogoutIcon />
        Logout</button
      >
    </div>
  </div>
{/if}
