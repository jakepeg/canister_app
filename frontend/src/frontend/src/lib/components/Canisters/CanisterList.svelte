<!-- frontend/src/frontend/src/lib/components/Canisters/CanisterList.svelte -->
<script lang="ts">
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/button/button.svelte"; // Corrected path
  import CanisterCard from "./CanisterCard.svelte";
  import { Principal } from "@dfinity/principal";

  // Define a mock CanisterInfo type for now
  type CanisterInfo = {
    id: string;
    name: string;
    iconUrl?: string; // Optional icon URL
  };

  // Svelte 5 Props
  type Props = {
    canisters: CanisterInfo[];
    onOpenCreateModal?: () => void; // Callback from parent (+page.svelte)
    onRefreshCanisters?: () => void; // Callback from parent (+page.svelte)
  };
  let {
    canisters = [],
    onOpenCreateModal,
    onRefreshCanisters,
  }: Props = $props();

  function handleOpenCreateModal() {
    if (onOpenCreateModal) {
      onOpenCreateModal();
    }
  }

  function navigateToCanisterFiles(canisterId: string) {
    // TODO: Confirm navigation path is correct
    goto(`/canister/${canisterId}/files`);
  }

  function handleCanisterCardUpdate() {
    // This is called when a CanisterCard signals an update (rename/delete)
    console.log(
      "CanisterList: Card signaled update. Calling onRefreshCanisters.",
    );
    if (onRefreshCanisters) {
      onRefreshCanisters(); // Call the callback passed from +page.svelte
    }
  }

  // Mock data for demonstration if needed (can be removed if parent passes data)
  // $: if (!canisters || canisters.length === 0) {
  //  canisters = [
  //      { id: '1', name: 'My First Canister' },
  //      { id: '2', name: 'Project Alpha' },
  //      { id: '3', name: 'Test Environment' }
  //  ];
  // }
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <!-- Header: Style: style_ESKRTZ - Inder, 20px, White -->
    <h1 class="font-inder text-xl">My Canisters</h1>
    <!-- New Canister Button: Style: style_GUBF0I - Inder, 17px, White, white stroke, 6px border-radius -->
    <Button onclick={handleOpenCreateModal}>New Canister</Button>
  </div>

  {#if canisters.length > 0}
    <!-- Canister Grid: Based on Figma 288:76 -->
    <div
      class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6"
    >
      {#each canisters as canister (canister.id)}
        <CanisterCard
          canisterId={Principal.fromText(canister.id)}
          initialCanisterName={canister.name}
          onClick={() => navigateToCanisterFiles(canister.id)}
          onUpdate={handleCanisterCardUpdate}
        />
      {/each}
    </div>
  {:else}
    <!-- Empty State: Based on Figma 280:6 -->
    <div class="flex flex-col items-center justify-center text-center py-16">
      <!-- Logo/Icon: Based on 280:30 -->
      <img src="/logo.svg" alt="Logo" class="w-24 h-24 mb-6 opacity-50" />
      <!-- Text: Style: style_8GQ93Y - Inder, 20px, White, Centered -->
      <p class="font-inder text-xl mb-6">Create a canister to get started.</p>
      <!-- Button is already present in the header -->
    </div>
  {/if}
</div>

<style>
  /* Ensure Inder font is loaded if not globally available */
  /* @import url('https://fonts.googleapis.com/css2?family=Inder&display=swap'); */
  .font-inder {
    font-family: "Inder", sans-serif;
  }
</style>
