<script lang="ts">
  import Upload from "$lib/components/Upload/Upload.svelte";
  import MultiUpload from "$lib/components/Upload/MultiUpload.svelte";
  import { page } from "$app/stores";
  import { authStore } from "$lib/services/auth";

  // let alias = $page.url.searchParams.get("alias");
</script>

{#if $authStore.state === "authenticated" || $authStore.state === "unauthenticated"}
  <!-- {#if alias?.startsWith("group-")} -->
  <MultiUpload auth={$authStore} />
  <!-- {:else}
    <Upload auth={$authStore} />
  {/if} -->
{/if}
