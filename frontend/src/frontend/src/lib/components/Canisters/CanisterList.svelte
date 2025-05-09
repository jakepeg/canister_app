<!-- frontend/src/frontend/src/lib/components/Canisters/CanisterList.svelte -->
<script lang="ts">
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/button/button.svelte"; // Corrected path
  import CanisterCard from "./CanisterCard.svelte";
  import { Principal } from "@dfinity/principal";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
  } from "$lib/components/ui/tooltip";

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
    hasSufficientFunds: boolean; // New prop
    balanceLoading: boolean;
  };
  let {
    canisters = [],
    onOpenCreateModal,
    onRefreshCanisters,
    hasSufficientFunds = false, // Default value
    balanceLoading = true, // Default value
  }: Props = $props();

  const insufficientFundsMessage =
    "To create a canister and fund it with cycles for computation, at least 1 ICP is required. To add ICP to your account, click the profile icon in the top right of the screen to open the dropdown. Copy your account id, purchase ICP on an exchange, and send 1 or more ICP to your account id.";

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
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <!-- Header: Style: style_ESKRTZ - Inder, 20px, White -->
    <h1 class="font-inder text-xl">My Canisters</h1>
    <!-- New Canister Button with tooltip when disabled -->
    {#if balanceLoading}
      <Button disabled class="opacity-75">Loading Balance...</Button>
    {:else}
      <Tooltip>
        <TooltipTrigger asChild>
          <div>
            <Button
              onclick={handleOpenCreateModal}
              disabled={!hasSufficientFunds}
            >
              New Canister
            </Button>
          </div>
        </TooltipTrigger>
        {#if !hasSufficientFunds}
          <TooltipContent
            class="max-w-md bg-yellow-50 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300 border border-yellow-400 dark:border-yellow-600 p-3 rounded-lg shadow-lg"
          >
            <p class="font-semibold">Insufficient Funds</p>
            <p class="text-sm">{insufficientFundsMessage}</p>
          </TooltipContent>
        {/if}
      </Tooltip>
    {/if}
  </div>

  {#if canisters.length > 0}
    <!-- Canister Grid: Based on Figma 288:76 -->
    <div
      class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-10"
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
    <!-- Empty State with dark/light mode responsive text color -->
    <div class="flex flex-col items-center justify-center text-center py-16">
      <!-- Logo/Icon: Based on 280:30 -->
      <img src="/logo.svg" alt="Logo" class="w-24 h-24 mb-6 opacity-50" />
      <!-- Text with responsive color -->
      <p class="font-inder text-xl mb-6 dark:text-white text-gray-900">
        Create a canister to get started.
      </p>
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
