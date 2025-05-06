<!-- CanisterCard.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Principal } from "@dfinity/principal";
  import {
    getCanisterStatus,
    type CanisterStatusInfo,
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

  export let canisterId: Principal;
  export let canisterName: string;
  export let onClick: () => void;
  export let onCanisterUpdated: () => void;

  let statusInfo: CanisterStatusInfo | null = null;
  let error: string | null = null;
  let menuOpen = false;

  // Dialog state
  let renameDialogOpen = false;
  let deleteDialogOpen = false;
  let newCanisterName = "";
  let isLoading = false;
  let dialogError = "";

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

  async function refreshStatus() {
    const result = await getCanisterStatus(canisterId, canisterName);
    if ("err" in result) {
      error = result.err;
    } else {
      statusInfo = result;
    }
  }

  async function handleRename() {
    if (!newCanisterName.trim()) {
      dialogError = "Name cannot be empty";
      return;
    }
    isLoading = true;
    dialogError = "";

    const result = await renameCanister(canisterId, newCanisterName);
    if ("ok" in result) {
      refreshStatus();
      onCanisterUpdated();
      renameDialogOpen = false;
      newCanisterName = "";
    } else {
      dialogError = result.err;
    }
    isLoading = false;
  }

  async function handleDelete() {
    isLoading = true;
    dialogError = "";

    const result = await deleteCanister(canisterId);
    if ("ok" in result) {
      onCanisterUpdated();
      deleteDialogOpen = false;
    } else {
      dialogError = result.err;
    }
    isLoading = false;
  }

  onMount(refreshStatus);
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
        {#if dialogError}
          <p class="text-red-500 text-sm">{dialogError}</p>
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
        Are you sure you want to delete "{statusInfo?.name || canisterName}"?
        This action cannot be undone.
      </p>
      {#if dialogError}
        <p class="text-red-500 text-sm mb-4">{dialogError}</p>
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

<div class="relative">
  <Card.Root
    class="w-full h-full border border-[#1F1F1F] shadow-[0px_4px_14px_2px_#0B8CE9] rounded-[15px] cursor-pointer"
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
            <MoreVertical class="w-3 h-[13px] text-white/75 cursor-pointer" />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content
            class="w-[134px] bg-[#1F1F1F] border border-[#0B8CE9] rounded-[11px] p-1 z-40 absolute right-0"
            sideOffset={5}
          >
            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
              onclick={() => {
                menuOpen = false;
                console.log("Start/Stop clicked");
              }}
            >
              <span>Start/Stop</span>
            </DropdownMenu.Item>

            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
              onclick={() => {
                menuOpen = false;
                console.log("Topup Cycles clicked");
              }}
            >
              <span>Topup Cycles</span>
            </DropdownMenu.Item>

            <DropdownMenu.Item
              class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
              onclick={() => {
                menuOpen = false;
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
        on:click={onClick}
        on:keydown={(e) => e.key === "Enter" && onClick()}
        class="h-full"
      >
        <!-- Canister Info -->
        <div class="mt-8 space-y-4">
          <!-- ID -->
          <div class="text-[11px] leading-[14px] text-white">
            ID: {truncateId(canisterId.toText())}
          </div>

          <!-- Cycles -->
          <div
            class="text-[11px] leading-[14px] text-white border-b border-white/20 pb-2"
          >
            {#if statusInfo}
              Cycles {formatCycles(statusInfo.cyclesBalance)} T
            {:else}
              Loading cycles...
            {/if}
          </div>

          <!-- Storage -->
          <div class="space-y-2 border-b border-white/20 pb-2">
            <div class="text-[11px] leading-[14px] text-white">Storage</div>
            {#if statusInfo}
              <div class="relative h-[7px] bg-[#B6C8CE] overflow-hidden">
                <div
                  class="absolute left-0 top-0 h-full bg-[#0B8CE9]"
                  style:width={`${(Number(statusInfo.memorySize) / Number(statusInfo.memoryAllocation)) * 100}%`}
                ></div>
              </div>
              <div class="text-[11px] leading-[14px] text-white">
                {formatGB(statusInfo.memorySize)} / {formatGB(
                  statusInfo.memoryAllocation,
                )} GB used
              </div>
            {:else}
              <div>Loading storage...</div>
            {/if}
          </div>

          <!-- Name -->
          <div class="text-[14px] leading-[18px] text-white text-center mt-2">
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
