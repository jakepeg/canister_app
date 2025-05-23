<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import Details from "$lib/components/Details/Details.svelte";
  import { authStore, authService } from "$lib/services/auth";

  $: {
    if ($authStore.state === "unauthenticated") {
      goto("/");
    }
  }

  // Extract canisterId from the route parameters
  $: canisterId = $page.params.canisterId;

  $: {
    if (canisterId) {
      authService.setCurrentCanister(canisterId);
    }
  }
</script>

{#if $authStore.state === "uninitialized"}
  <h1 class="title-1">Loading...</h1>
{:else if $authStore.state === "authenticated"}
  <Details auth={$authStore} />
{/if}
