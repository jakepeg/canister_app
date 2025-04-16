<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores"; // Import page store to access params
  import RequestsList from "$lib/components/Requests/RequestsList.svelte";
  import { authStore, authService } from "$lib/services/auth"; // Import authService

  // Get canisterId from the route parameters
  let canisterId: string | undefined;
  $: canisterId = $page.params.canisterId;

  // Set the current canister in the auth service when the ID changes
  $: {
    if (canisterId) {
      authService.setCurrentCanister(canisterId); // Set context here
    }
  }

  $: {
    // Redirect if not authenticated or if canisterId is missing (shouldn't happen with valid routing)
    if ($authStore.state === "unauthenticated" || !canisterId) {
      goto("/");
    }
  }
</script>

{#if $authStore.state === "uninitialized"}
  <h1 class="title-1">Loading...</h1>
{:else if $authStore.state === "authenticated" && canisterId}
  <!-- Pass canisterId to the RequestsList component -->
  <RequestsList auth={$authStore} />
{:else if !canisterId}
  <h1 class="title-1 text-destructive">Error: Canister ID not found in URL.</h1>
{/if}
