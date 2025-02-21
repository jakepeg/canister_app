<script>
  import TestEncryptedUpload from "$lib/components/Test/TestEncryptedUpload.svelte";
  import { authStore } from "$lib/services/auth";
</script>

{#if $authStore.state === "uninitialized"}
  <h1 class="title-1">Loading...</h1>
{:else if $authStore.state === "authenticated"}
  <TestEncryptedUpload auth={$authStore} />
{:else}
  Login in dum dum
{/if}
