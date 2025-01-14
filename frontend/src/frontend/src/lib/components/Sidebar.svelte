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
      <a
        href="/"
        class="btn btn-ghost w-full justify-start {$page.route.id === '/'
          ? 'btn-ghost-active'
          : ''}"
      >
        <IconFile />
        Files
      </a>
      <a
        href="/upload"
        class="btn btn-ghost w-full justify-start {$page.route.id === '/upload'
          ? 'btn-ghost-active'
          : ''}"
      >
        <UploadIcon />
        Upload
      </a>
      <a
        href="/requests"
        class="btn btn-ghost w-full justify-start {$page.route.id ===
        '/requests'
          ? 'btn-ghost-active'
          : ''}"
      >
        <RequestsIcon />
        Requests
      </a>
    </nav>

    <!-- <div class="mt-auto">
      <button on:click={logout} class="btn btn-ghost w-full justify-start">
        <LogoutIcon />
        Logout
      </button>
    </div> -->
  </div>
</aside>
