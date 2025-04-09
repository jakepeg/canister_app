<script lang="ts">
  import { goto } from "$app/navigation";
  import PersonalSolutions from "$lib/components/Solutions/Personal.svelte";
  import { authStore } from "$lib/services/auth";

  $: {
    if ($authStore.state === "authenticated") {
      goto("/");
    }
  }
</script>

{#if $authStore.state === "uninitialized"}
  <h1 class="title-1">Loading...</h1>
{:else if $authStore.state === "unauthenticated"}
  <PersonalSolutions />
{/if}
