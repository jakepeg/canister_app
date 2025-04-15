<script>
  import Balance from "$lib/components/User/Balance.svelte";
  import { authStore } from "$lib/services/auth";
</script>

{#if $authStore.state === "uninitialized"}
  Auth uninitialized
{:else if $authStore.state === "authenticated"}
  <Balance/>
{:else}
  Login in dum dum
{/if}
