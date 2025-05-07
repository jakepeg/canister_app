<!-- frontend/src/frontend/src/routes/canister/[canisterId]/files/+page.svelte -->
<script lang="ts">
  import { page } from "$app/stores";
  import FileList from "$lib/components/Home/FileList.svelte"; // Assuming this is the correct path
  import NotAuthenticated from "$lib/components/Home/NotAuthenticated.svelte";
  import { authStore, authService } from "$lib/services/auth";
  import { onMount } from "svelte";

  // export const prerender = false;

  // Extract canisterId from the route parameters
  $: canisterId = $page.params.canisterId;

  $: {
    if (canisterId) {
      authService.setCurrentCanister(canisterId);
    }
  }
</script>

<section class="w-full">
  {#if $authStore.state === "uninitialized"}
    <div class="flex justify-center items-center h-screen">
      <h1 class="text-xl text-white">Loading...</h1>
    </div>
  {:else if $authStore.state === "authenticated"}
    <!-- Display the FileList for the specific canister -->
    <!-- Add a title or breadcrumb indicating the current canister -->
    <h1 class="text-lg mb-4 px-4 pt-4">Files in Canister: {canisterId}</h1>
    <FileList auth={$authStore} />
  {:else}
    <NotAuthenticated />
  {/if}
</section>
