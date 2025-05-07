<script lang="ts">
  import { page } from "$app/stores";
  import { authService, authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import IconFile from "./icons/IconFile.svelte";
  import LogoIcon from "./icons/LogoIcon.svelte";
  import LogoutIcon from "./icons/LogoutIcon.svelte";
  import RequestsIcon from "./icons/RequestsIcon.svelte";
  import UploadIcon from "./icons/UploadIcon.svelte";
  import { uploadInProgress } from "$lib/services/upload";
  import * as Sidebar from "$lib/components/ui/sidebar";

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

<aside class="bg-background w-64 min-h-screen flex flex-col">
  <div class="p-4">
    <nav class="flex-1">
      <!-- Files link - dynamic based on canisterId param -->
      <a
        href={$page.params.canisterId
          ? `/canister/${$page.params.canisterId}/files`
          : "/"}
        class="btn btn-ghost w-full justify-start {$page.route.id ===
        '/canister/[canisterId]/files'
          ? 'btn-ghost-active'
          : ''}"
        aria-current={$page.route.id === '/canister/[canisterId]/files' ? 'page' : undefined}
      >
        <IconFile />
        Files
      </a>
      {#if $authStore.state === "authenticated"}
        <!-- Requests link - dynamic based on canisterId param -->
        <!-- Fallback to root if no canisterId, though requests might not make sense there -->
        <a
          href={$page.params.canisterId
            ? `/canister/${$page.params.canisterId}/requests`
            : "/"}
          class="btn btn-ghost w-full justify-start {$page.route.id ===
          '/canister/[canisterId]/requests'
            ? 'btn-ghost-active'
            : ''}"
          aria-current={$page.route.id === '/canister/[canisterId]/requests' ? 'page' : undefined}
        >
          <RequestsIcon />
          Requests
        </a>
      {/if}
    </nav>

    <!-- <div class="mt-auto">
      <button on:click={logout} class="btn btn-ghost w-full justify-start">
        <LogoutIcon />
        Logout
      </button>
    </div> -->
  </div>
</aside>
