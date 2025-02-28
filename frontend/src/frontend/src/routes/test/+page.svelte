<script>
  import TestEncryptedUpload from "$lib/components/Test/TestEncryptedUpload.svelte";
  import { authStore } from "$lib/services/auth";
</script>

{#if $authStore.state === "uninitialized"}
  Auth uninitialized
{:else if $authStore.state === "authenticated"}
  <TestEncryptedUpload auth={$authStore} />
{:else}
  Login in dum dum
{/if}
