<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    renameCanister,
    deleteCanister,
    startUserCanister, // Import new functions
    stopUserCanister, // Import new functions
    type CanisterStatusInfo,
  } from "$lib/services/canisterManagement";
  import type { Principal } from "@dfinity/principal";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import { enumIs } from "$lib/shared/enums";

  type Props = {
    canisterId: Principal;
    canisterName: string;
    canisterCurrentStatus?: CanisterStatusInfo["status"]; // New optional prop for current status
    onCanisterRenamed?: () => void; // Callback prop
    onCanisterDeleted?: () => void; // Callback prop
    onCanisterStatusChanged?: () => void; // New callback for status changes
  };
  let {
    canisterId,
    canisterName,
    canisterCurrentStatus,
    onCanisterRenamed,
    onCanisterDeleted,
    onCanisterStatusChanged,
  }: Props = $props();

  let renameDialogOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let newCanisterName = $state("");
  let isLoading = $state(false);
  let error = "";
  let actionError = $state(""); // Renamed from 'error' to be more specific

  // Computed properties for Start/Stop button visibility/state
  // These will determine if the canister is running or stopped.
  // If canisterCurrentStatus is undefined, we might disable or hide the buttons.
  let isRunning = $derived(
    canisterCurrentStatus && "running" in canisterCurrentStatus,
  );
  let isStopped = $derived(
    canisterCurrentStatus && "stopped" in canisterCurrentStatus,
  );
  let isStopping = $derived(
    canisterCurrentStatus && "stopping" in canisterCurrentStatus,
  );

  function handleRename() {
    // In a real app, you'd open a rename dialog or call a service
    console.log(`Simulating rename for ${canisterId.toText()}`);
    if (onCanisterRenamed) {
      onCanisterRenamed();
    }
  }

  function handleDelete() {
    // In a real app, you'd show a confirmation and call a service
    console.log(`Simulating delete for ${canisterId.toText()}`);
    if (onCanisterDeleted) {
      onCanisterDeleted();
    }
  }

  async function handleStartCanister() {
    isLoading = true;
    actionError = "";
    console.log(`Attempting to start canister: ${canisterId.toText()}`);
    const result = await startUserCanister(canisterId);
    if (enumIs(result, "err")) {
      actionError = `Start error: ${result.err}`;
      // Display this error to the user, perhaps in a toast or dedicated error area
      console.error(actionError);
    } else {
      console.log("Start successful, calling onCanisterStatusChanged");
      if (onCanisterStatusChanged) {
        onCanisterStatusChanged();
      }
    }
    isLoading = false;
  }

  async function handleStopCanister() {
    isLoading = true;
    actionError = "";
    console.log(`Attempting to stop canister: ${canisterId.toText()}`);
    const result = await stopUserCanister(canisterId);
    if (enumIs(result, "err")) {
      actionError = `Stop error: ${result.err}`;
      // Display this error
      console.error(actionError);
    } else {
      console.log("Stop successful, calling onCanisterStatusChanged");
      if (onCanisterStatusChanged) {
        onCanisterStatusChanged();
      }
    }
    isLoading = false;
  }
</script>

<!-- Rename Dialog -->
<Dialog.Root bind:open={renameDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] bg-[#1F1F1F] p-6 shadow-lg rounded-[21px] font-inder"
    >
      <Dialog.Title class="text-lg mb-4">Rename Canister</Dialog.Title>
      <div class="space-y-4">
        <Input
          type="text"
          placeholder="Enter new name"
          bind:value={newCanisterName}
          class="bg-transparent border border-[#0B8CE9] rounded-[9px]"
        />
        {#if error}
          <p class="text-red-500 text-sm">{error}</p>
        {/if}
        <div class="flex justify-end gap-2">
          <Button
            variant="outline"
            onclick={() => (renameDialogOpen = false)}
            disabled={isLoading}
          >
            Cancel
          </Button>
          <Button variant="outline" onclick={handleRename} disabled={isLoading}>
            {isLoading ? "Renaming..." : "Rename"}
          </Button>
        </div>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] bg-[#1F1F1F] p-6 shadow-lg rounded-[21px] font-inder"
    >
      <Dialog.Title class="text-lg mb-4">Delete Canister</Dialog.Title>
      <p class="mb-4">
        Are you sure you want to delete "{canisterName}"? This action cannot be
        undone.
      </p>
      {#if error}
        <p class="text-red-500 text-sm mb-4">{error}</p>
      {/if}
      <div class="flex justify-end gap-2">
        <Button
          variant="outline"
          onclick={() => (deleteDialogOpen = false)}
          disabled={isLoading}
        >
          Cancel
        </Button>
        <Button
          variant="outline"
          class="text-red-500"
          onclick={handleDelete}
          disabled={isLoading}
        >
          {isLoading ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<DropdownMenu.Content
  class="w-[134px] bg-[#1F1F1F] border border-[#0B8CE9] rounded-[11px] p-1 z-50"
  sideOffset={5}
  align="end"
>
  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
    on:click={() => (renameDialogOpen = true)}
  >
    Rename
  </DropdownMenu.Item>

  <!-- Start/Stop Canister Items -->
  {#if canisterCurrentStatus}
    {#if isStopped || isStopping}
      <!-- Show Start if stopped or stopping (stopping implies it will be stopped) -->
      <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        onclick={handleStartCanister}
        disabled={isLoading || isStopping}
      >
        {#if isLoading && !renameDialogOpen && !deleteDialogOpen}Starting...{:else}Start
          Canister{/if}
      </DropdownMenu.Item>
    {/if}
    {#if isRunning}
      <!-- Show Stop if running -->
      <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        onclick={handleStopCanister}
        disabled={isLoading}
      >
        {#if isLoading && !renameDialogOpen && !deleteDialogOpen}Stopping...{:else}Stop
          Canister{/if}
      </DropdownMenu.Item>
    {/if}
  {:else}
    <DropdownMenu.Item
      class="relative flex items-center rounded-sm px-2 py-1.5 text-sm text-gray-500 font-inder"
      disabled={true}
    >
      Start/Stop (No Status)
    </DropdownMenu.Item>
  {/if}

  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
    on:click={() => console.log("Backup clicked")}
  >
    Backup
  </DropdownMenu.Item>

  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
    on:click={() => console.log("Controllers clicked")}
  >
    Controllers
  </DropdownMenu.Item>

  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
    on:click={() => console.log("Topup Cycles clicked")}
  >
    Topup Cycles
  </DropdownMenu.Item>

  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
    on:click={() => console.log("Memory clicked")}
  >
    Memory
  </DropdownMenu.Item>

  <DropdownMenu.Separator class="h-px bg-[#0B8CE9] my-1" />

  <DropdownMenu.Item
    class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-red-500 font-inder"
    on:click={() => (deleteDialogOpen = true)}
  >
    Delete
  </DropdownMenu.Item>
</DropdownMenu.Content>
