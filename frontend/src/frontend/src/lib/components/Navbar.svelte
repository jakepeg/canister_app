<script lang="ts">
  import { page } from "$app/stores";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import LogoIcon from "./icons/LogoIcon.svelte";
  import LogoutIcon from "./icons/LogoutIcon.svelte";
  import AccountIcon from "./icons/AccountIcon.svelte";
  import RequestsIcon from "./icons/RequestsIcon.svelte";
  import UploadIcon from "./icons/UploadIcon.svelte";
  import { uploadInProgress } from "$lib/services/upload";
  import ModeToggle from "$lib/components/mode-toggle.svelte";
  import Balance from "$lib/components/User/Balance.svelte";
  import { Button } from "$lib/components/ui/button";
  import { fade, fly } from "svelte/transition";
  import IconFile from "./icons/IconFile.svelte";

  let showMobileMenu = false;
  let showBalance = false;

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

<nav
  class="w-full relative z-20"
  class:bg-sidebar={$authStore.state === "authenticated"}
  class:bg-transparent={$authStore.state === "unauthenticated"}
  class:backdrop-blur={$authStore.state === "unauthenticated"}
>
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
    <div class="flex items-center ml-auto gap-1">
      {#if $authStore.state === "authenticated"}
        <ModeToggle />
        <div class="relative">
          <button
            onclick={() => (showBalance = !showBalance)}
            class="btn btn-ghost"
          >
            <AccountIcon />
          </button>

          {#if showBalance}
            <div
              role="button"
              class="absolute right-0 top-[67px] mt-0"
              transition:fade={{ duration: 100 }}
              onmouseleave={() => (showBalance = false)}
            >
              <Balance />
            </div>
          {/if}
        </div>
      {/if}

      <!-- Simplified Login/Logout Logic -->
      {#if $authStore.state === "unauthenticated"}
        <div class="hidden md:flex items-center gap-4">
          <a
            href="/personal"
            class="font-bold transition-colors"
            class:text-blue-400={$page.url.pathname.startsWith("/personal")}
            class:text-white={!$page.url.pathname.startsWith("/personal")}
            class:hover:text-blue-400={!$page.url.pathname.startsWith(
              "/personal",
            )}
          >
            Personal
          </a>

          <a
            href="/enterprise"
            class="font-bold transition-colors"
            class:text-blue-400={$page.url.pathname.startsWith("/enterprise")}
            class:text-white={!$page.url.pathname.startsWith("/enterprise")}
            class:hover:text-blue-400={!$page.url.pathname.startsWith(
              "/enterprise",
            )}
          >
            Enterprise
          </a>

          <!-- <a
            href="/faq"
            class="font-bold transition-colors"
            class:text-blue-400={$page.url.pathname.startsWith("/faq")}
            class:text-white={!$page.url.pathname.startsWith("/faq")}
            class:hover:text-blue-400={!$page.url.pathname.startsWith("/faq")}
          >
            FAQ
          </a> -->

          <button
            class="gap-4 btn btn-accent"
            onclick={() => authService.login()}
          >
            <LogoIcon />
            Login
          </button>
        </div>
      {:else if $authStore.state === "authenticated"}
        <!-- Hamburger menu (Mobile Only) -->
        <button
          class="flex flex-col items-stretch gap-[5px] md:hidden w-5 h-5"
          onclick={() => (showMobileMenu = !showMobileMenu)}
          aria-label="Open menu"
          aria-expanded={showMobileMenu}
        >
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-transform duration-200 {showMobileMenu
              ? 'rotate-45 translate-y-[7px]'
              : 'rotate-0'}"
          ></span>
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-opacity duration-200 {showMobileMenu
              ? 'opacity-0'
              : 'opacity-100'}"
          ></span>
          <span
            class="h-[2px] bg-accent-100 rounded-full transition-transform duration-200 {showMobileMenu
              ? '-rotate-45 translate-y-[-7px]'
              : 'rotate-0'}"
          ></span>
        </button>

        <!-- Desktop Logout Button -->
        <div class="hidden md:flex gap-2 lg:gap-8">
          <button
            onclick={() => logout()}
            class="btn btn-ghost p-2"
            aria-label="Logout"
          >
            <!-- Added padding -->
            <LogoutIcon />
          </button>
        </div>

        <!-- REMOVED the seemingly erroneous "Demo" button from here -->
      {/if}
    </div>
  </div>
</nav>

{#if showMobileMenu}
  <div
    class="md:hidden fixed inset-0 bg-black/50"
    transition:fade|global={{ duration: 200 }}
  ></div>
  <div
    transition:fly|global={{ duration: 300, x: 1000, opacity: 1 }}
    class="fixed md:hidden inset-0 bg-background-300 z-10 pt-16"
  >
    <div class="p-4 flex flex-col gap-4 h-full">
      <a
        href="/"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/"}
        onclick={() => (showMobileMenu = false)}
      >
        <IconFile />
        Files</a
      >
      <a
        href="/upload"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/upload"}
        onclick={() => (showMobileMenu = false)}
      >
        <UploadIcon />
        Upload</a
      >
      <a
        href="/requests"
        class="btn btn-ghost justify-start"
        class:btn-ghost-active={$page.route.id === "/requests"}
        onclick={() => (showMobileMenu = false)}
      >
        <RequestsIcon />
        Requests</a
      >
      <div class="flex-1"></div>
      <button
        onclick={() => {
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
