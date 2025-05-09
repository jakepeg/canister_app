<!-- CanisterCard.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Principal } from "@dfinity/principal";
  import {
    getCanisterStatus,
    type CanisterStatusInfo,
    startUserCanister, // Import new functions
    stopUserCanister, // Import new functions
  } from "$lib/services/canisterManagement";
  import * as Card from "$lib/components/ui/card";
  import { MoreVertical } from "lucide-svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import {
    renameCanister,
    deleteCanister,
  } from "$lib/services/canisterManagement";
  import TopUpCyclesModal from "./TopUpCyclesModal.svelte"; // Import the new modal

  // Svelte 5 Props
  type Props = {
    canisterId: Principal;
    initialCanisterName: string; // Renamed for clarity, passed from list
    onClick: () => void;
    onUpdate?: () => void; // Callback to notify parent (CanisterList) of changes
  };
  let { canisterId, initialCanisterName, onClick, onUpdate }: Props = $props();

  // Svelte 5 Runes for state
  let statusInfo = $state<CanisterStatusInfo | null>(null);
  let cardError = $state<string | null>(null); // Error specific to fetching status for this card
  let menuOpen = $state(false);

  // Dialog state - specific to this card's modals
  let renameDialogOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let newNameInput = $state(initialCanisterName); // Pre-fill with current name
  let isDialogLoading = $state(false);
  let dialogError = $state("");

  // State for Start/Stop functionality
  let isStartStopLoading = $state(false);
  let startStopError = $state<string | null>(null); // Specific error for start/stop actions

  let topUpModalOpen = $state(false); // State for the new modal

  // Helper function to get status color
  function getStatusColor(status: CanisterStatusInfo["status"]): string {
    if ("running" in status) return "#25C51C";
    if ("stopping" in status) return "#FFA500";
    return "#FF0000"; // stopped
  }

  // Helper function to get status text
  function getStatusText(status: CanisterStatusInfo["status"]): string {
    if ("running" in status) return "Running";
    if ("stopping" in status) return "Stopping";
    return "Stopped";
  }

  // Format bytes to GB with 1 decimal place
  function formatGB(bytes: bigint): string {
    const gb = Number(bytes) / (1024 * 1024 * 1024);
    return gb.toFixed(1);
  }

  // Format cycles to T (trillions)
  function formatCycles(cycles: bigint): string {
    const t = Number(cycles) / 1_000_000_000_000;
    return t.toFixed(3);
  }

  // Truncate canister ID for display
  function truncateId(id: string): string {
    return `${id.slice(0, 5)}...${id.slice(-7)}`;
  }

  async function refreshCardStatus() {
    cardError = null;
    // Use the most current known name for fetching status, which might be statusInfo.name or initialCanisterName
    const nameToFetch = statusInfo?.name || initialCanisterName;
    console.log(
      `CanisterCard (${canisterId.toText()}): Refreshing status with name: ${nameToFetch}`,
    );
    const result = await getCanisterStatus(canisterId, nameToFetch);
    if ("err" in result) {
      cardError = result.err;
      // statusInfo = null; // Optionally clear, or keep old data on error
      console.error(
        `CanisterCard (${canisterId.toText()}): Error refreshing status:`,
        result.err,
      );
    } else {
      statusInfo = result;
      console.log(
        `CanisterCard (${canisterId.toText()}): Status refreshed:`,
        statusInfo,
      );
    }
  }

  function openRenameDialog() {
    newNameInput = statusInfo?.name || initialCanisterName; // Pre-fill with the latest known name
    dialogError = "";
    renameDialogOpen = true;
    menuOpen = false; // Close dropdown
  }

  async function submitRename() {
    if (!newNameInput.trim()) {
      dialogError = "Name cannot be empty";
      return;
    }
    isDialogLoading = true;
    dialogError = "";

    console.log(
      `CanisterCard (${canisterId.toText()}): Attempting to rename to "${newNameInput}"`,
    );
    const result = await renameCanister(canisterId, newNameInput); // Backend call

    if ("ok" in result) {
      console.log(
        `CanisterCard (${canisterId.toText()}): Rename successful. New name: "${newNameInput}"`,
      );
      // The backend has confirmed the rename.
      // 1. Update local `statusInfo` if possible, or rely on full refresh.
      //    For immediate UI update of the name on THIS card:
      if (statusInfo) {
        statusInfo.name = newNameInput; // Directly update the displayed name
      }
      // 2. Close the dialog.
      renameDialogOpen = false;
      // 3. Call `onUpdate` to tell the parent list to re-fetch all canisters.
      //    This ensures the list in `+page.svelte` gets the absolute latest from the backend.
      if (onUpdate) {
        onUpdate();
      }
      // 4. Optionally, refresh this card's full status again (might be redundant if onUpdate re-fetches everything)
      // await refreshCardStatus(); // This will get the new name again from canister status
    } else {
      dialogError = result.err;
      console.error(
        `CanisterCard (${canisterId.toText()}): Rename failed:`,
        result.err,
      );
    }
    isDialogLoading = false;
  }

  async function submitDelete() {
    isDialogLoading = true;
    dialogError = "";
    console.log(`CanisterCard (${canisterId.toText()}): Attempting to delete.`);
    const result = await deleteCanister(canisterId); // Backend call

    if ("ok" in result) {
      console.log(`CanisterCard (${canisterId.toText()}): Delete successful.`);
      deleteDialogOpen = false;
      // Tell the parent list to re-fetch, which will remove this card.
      if (onUpdate) {
        onUpdate();
      }
    } else {
      dialogError = result.err;
      console.error(
        `CanisterCard (${canisterId.toText()}): Delete failed:`,
        result.err,
      );
    }
    isDialogLoading = false;
  }

  // Handler for Start/Stop Canister
  async function handleStartStop() {
    if (!statusInfo || isStartStopLoading || "stopping" in statusInfo.status) {
      console.log("Disabled or loading, not attempting start/stop.");
      // Do nothing if no status, already loading, or canister is in "stopping" state
      return;
    }

    isStartStopLoading = true;
    startStopError = null; // Clear previous error
    menuOpen = false; // Close dropdown

    let result;
    try {
      if ("running" in statusInfo.status) {
        console.log(
          `CanisterCard (${canisterId.toText()}): Attempting to stop.`,
        );
        result = await stopUserCanister(canisterId);
      } else if ("stopped" in statusInfo.status) {
        console.log(
          `CanisterCard (${canisterId.toText()}): Attempting to start.`,
        );
        result = await startUserCanister(canisterId);
      } else {
        // Should not happen if button is enabled correctly
        console.warn(
          `CanisterCard (${canisterId.toText()}): No action for status ${getStatusText(statusInfo.status)}`,
        );
        isStartStopLoading = false;
        return;
      }

      if (result && "ok" in result) {
        console.log(
          `CanisterCard (${canisterId.toText()}): Start/Stop operation successful.`,
        );
        // Status will be updated by refreshCardStatus
      } else if (result && "err" in result) {
        startStopError = result.err;
        console.error(
          `CanisterCard (${canisterId.toText()}): Start/Stop operation failed: ${result.err}`,
        );
      }
    } catch (e: any) {
      startStopError =
        e.message || "An unexpected error occurred during start/stop.";
      console.error(
        `CanisterCard (${canisterId.toText()}): Unexpected Start/Stop error:`,
        e,
      );
    } finally {
      isStartStopLoading = false;
      await refreshCardStatus(); // Always refresh status afterwards
    }
  }

  // Handle Svelte 5 dialog open/close props
  function handleRenameDialogValidOpenChange(value: boolean) {
    renameDialogOpen = value;
    if (!value) dialogError = ""; // Clear error when closing
  }
  function handleDeleteDialogValidOpenChange(value: boolean) {
    deleteDialogOpen = value;
    if (!value) dialogError = ""; // Clear error when closing
  }

  // Functions for TopUpCyclesModal
  function openTopUpModal() {
    startStopError = null;
    cardError = null;
    menuOpen = false;
    topUpModalOpen = true;
  }

  function handleTopUpSuccess(event: CustomEvent<{ message: string }>) {
    console.log(
      "CanisterCard: Top-up success reported by modal:",
      event.detail.message,
    );
    // topUpModalOpen = false; // Modal will self-close on success or user clicks Done
    refreshCardStatus();
    // Optionally show a global success toast here
  }

  function handleTopUpModalClose() {
    topUpModalOpen = false; // Sync state if modal closes itself
  }

  onMount(() => {
    refreshCardStatus();
    newNameInput = initialCanisterName; // Ensure newNameInput is set on mount
  });
</script>

<!-- Rename Dialog -->
<Dialog.Root bind:open={renameDialogOpen}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] dark:bg-[#1F1F1F] bg-white p-6 shadow-lg rounded-[21px] font-inder dark:text-white text-gray-900"
    >
      <Dialog.Title class="text-lg mb-4">Rename Canister</Dialog.Title>
      <div class="space-y-4">
        <Input
          type="text"
          placeholder="Enter new name"
          bind:value={newNameInput}
          class="bg-transparent border border-[#0B8CE9] rounded-[9px]"
        />
        {#if dialogError}
          <p class="text-red-500 text-sm">{dialogError}</p>
        {/if}
        <div class="flex justify-end gap-2">
          <Button
            variant="outline"
            onclick={() => (renameDialogOpen = false)}
            disabled={isDialogLoading}
          >
            Cancel
          </Button>
          <Button
            variant="outline"
            onclick={submitRename}
            disabled={isDialogLoading}
          >
            {isDialogLoading ? "Renaming..." : "Rename"}
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
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] dark:bg-[#1F1F1F] bg-white p-6 shadow-lg rounded-[21px] font-inder dark:text-white text-gray-900"
    >
      <Dialog.Title class="text-lg mb-4">Delete Canister</Dialog.Title>
      <p class="mb-4">
        Are you sure you want to delete "{statusInfo?.name ||
          initialCanisterName}"? This action cannot be undone.
      </p>
      {#if dialogError}
        <p class="text-red-500 text-sm mb-4">{dialogError}</p>
      {/if}
      <div class="flex justify-end gap-2">
        <Button
          variant="outline"
          onclick={() => (deleteDialogOpen = false)}
          disabled={isDialogLoading}
        >
          Cancel
        </Button>
        <Button
          variant="outline"
          class="text-red-500"
          onclick={submitDelete}
          disabled={isDialogLoading}
        >
          {isDialogLoading ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<!-- Top Up Cycles Modal -->
{#if statusInfo && topUpModalOpen}
  <!-- Conditionally render or pass open prop, bind:open handles visibility -->
  <TopUpCyclesModal
    bind:open={topUpModalOpen}
    {canisterId}
    canisterName={statusInfo.name || initialCanisterName}
    currentCyclesT={formatCycles(statusInfo.cyclesBalance) + " T"}
    on:close={handleTopUpModalClose}
    on:topupSuccess={handleTopUpSuccess}
  />
{/if}

<div class="relative">
  <Card.Root
    class="w-full h-full border dark:border-[#1F1F1F] border-gray-200 shadow-[0px_4px_14px_2px_#0B8CE9] rounded-[15px] cursor-pointer dark:bg-[#1F1F1F] bg-white"
  >
    <Card.Content class="p-4 relative">
      <!-- Status Indicator -->
      {#if statusInfo}
        <div
          class="absolute w-[10px] h-[10px] left-[15px] top-[17px] rounded-full filter blur-[2px]"
          style:background-color={getStatusColor(statusInfo.status)}
          title={getStatusText(statusInfo.status)}
        ></div>
      {/if}

      <!-- Options dropdown -->
      <div class="absolute right-3 top-3">
        <DropdownMenu.Root bind:open={menuOpen}>
          <DropdownMenu.Trigger class="focus:outline-none">
            <MoreVertical
              class="w-3 h-[13px] dark:text-white/75 text-gray-600 cursor-pointer"
            />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content
            class="w-[134px] dark:bg-[#1F1F1F] bg-white border border-[#0B8CE9] rounded-[11px] p-1 z-40 absolute right-0"
            sideOffset={5}
          >
            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] dark:text-white text-gray-900 font-inder"
              disabled={!statusInfo ||
                isStartStopLoading ||
                (statusInfo && "stopping" in statusInfo.status)}
              onclick={handleStartStop}
            >
              {#if isStartStopLoading}
                <span>Processing...</span>
              {:else if statusInfo}
                {#if "running" in statusInfo.status}
                  <span>Stop Canister</span>
                {:else if "stopped" in statusInfo.status}
                  <span>Start Canister</span>
                {:else if "stopping" in statusInfo.status}
                  <span class="opacity-50">Stopping...</span>
                {:else}
                  <!-- Should not happen with valid statusInfo -->
                  <span class="opacity-50">Status Error</span>
                {/if}
              {:else}
                <span class="opacity-50">Status N/A</span>
              {/if}
            </DropdownMenu.Item>

            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] dark:text-white text-gray-900 font-inder"
              disabled={!statusInfo || isStartStopLoading}
              onclick={openTopUpModal}
            >
              <span>Topup Cycles</span>
            </DropdownMenu.Item>

            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] dark:text-white text-gray-900 font-inder"
              onclick={() => {
                menuOpen = true;
                console.log("Rename clicked");

                renameDialogOpen = true;
              }}
            >
              <span>Rename</span>
            </DropdownMenu.Item>

            <DropdownMenu.Separator class="h-px bg-[#0B8CE9] my-1" />

            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-red-500 font-inder"
              onclick={() => {
                menuOpen = false;
                deleteDialogOpen = true;
              }}
            >
              <span>Delete</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>

      <!-- Card content (clickable) -->
      <div
        role="button"
        tabindex="0"
        onclick={onClick}
        onkeydown={(e) => e.key === "Enter" && onClick()}
        class="h-full"
      >
        <!-- Canister Info -->
        <div class="mt-8 space-y-4">
          <!-- ID -->
          <div class="text-[11px] leading-[14px] dark:text-white text-gray-900">
            ID: {truncateId(canisterId.toText())}
          </div>

          <!-- Cycles -->
          <div
            class="text-[11px] leading-[14px] dark:text-white text-gray-900 border-b dark:border-white/20 border-gray-300 pb-2"
          >
            {#if statusInfo}
              Cycles {formatCycles(statusInfo.cyclesBalance)} T
            {:else}
              Loading cycles...
            {/if}
          </div>

          <!-- Storage -->
          <div
            class="space-y-2 border-b dark:border-white/20 border-gray-300 pb-2"
          >
            <div
              class="text-[11px] leading-[14px] dark:text-white text-gray-900"
            >
              Storage
            </div>
            {#if statusInfo}
              <div class="relative h-[7px] bg-[#B6C8CE] overflow-hidden">
                <div
                  class="absolute left-0 top-0 h-full bg-[#0B8CE9]"
                  style:width={`${(Number(statusInfo.memorySize) / Number(statusInfo.memoryAllocation)) * 100}%`}
                ></div>
              </div>
              <div
                class="text-[11px] leading-[14px] dark:text-white text-gray-900"
              >
                {formatGB(statusInfo.memorySize)} / {formatGB(
                  statusInfo.memoryAllocation,
                )} GB used
              </div>
            {:else}
              <div class="dark:text-white text-gray-900">
                Loading storage...
              </div>
            {/if}
          </div>

          <!-- Name -->
          <div
            class="text-[14px] leading-[18px] dark:text-white text-gray-900 text-center mt-2"
          >
            {#if statusInfo}
              {statusInfo.name}
            {:else}
              Loading...
            {/if}
          </div>
        </div>
      </div>
    </Card.Content>
  </Card.Root>
</div>
