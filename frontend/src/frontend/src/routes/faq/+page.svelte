<script lang="ts">
  import FAQ from "$lib/components/FAQ.svelte";
  import { authStore } from "$lib/services/auth";
</script>

{#if $authStore.state === "uninitialized"}
  <h1 class="title-1">Loading...</h1>
{:else}
  <FAQ />
{/if}
