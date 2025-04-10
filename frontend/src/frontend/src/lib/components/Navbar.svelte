<script lang="ts">
  import { page } from "$app/stores";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import LogoIcon from "./icons/LogoIcon.svelte";
  import LogoutIcon from "./icons/LogoutIcon.svelte";
  import { uploadInProgress } from "$lib/services/upload";
  import ModeToggle from "$lib/components/mode-toggle.svelte";

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
    <div class="flex items-center ml-auto">
      {#if $authStore.state === "authenticated"}
        <ModeToggle />
      {/if}

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

          <button
            class="gap-4 btn btn-accent"
            on:click={() => authService.login()}
          >
            <LogoIcon />
            Demo
          </button>
        </div>
      {:else if $authStore.state === "authenticated"}
        <div class="hidden md:flex gap-2 lg:gap-8">
          <button on:click={() => logout()} class="btn btn-ghost">
            <LogoutIcon />
          </button>
        </div>
      {/if}
    </div>
  </div>
</nav>
