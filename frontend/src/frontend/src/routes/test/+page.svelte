<script>
  import TestEncryptedUpload from "$lib/components/Test/TestEncryptedUpload.svelte";
  import { authStore } from "$lib/services/auth";
</script>

{#if $authStore.state === "uninitialized"}
  <TestEncryptedUpload />
{:else if $authStore.state === "authenticated"}
  <TestEncryptedUpload />
{:else}
  Login in dum dum
{/if}
