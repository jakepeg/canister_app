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
  import CanisterOptionsMenu from "./CanisterOptionsMenu.svelte";

  export let canisterId: Principal;
  export let canisterName: string;
  export let onClick: () => void;
  export let onCanisterUpdated: () => void;

  let statusInfo: CanisterStatusInfo | null = null;
  let error: string | null = null;
  let menuOpen = false;

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

  function handleCanisterRenamed() {
    refreshStatus();
    onCanisterUpdated();
  }

  function handleCanisterDeleted() {
    onCanisterUpdated();
  }

  onMount(refreshStatus);
</script>

<div class="w-[172px] h-[185px] relative">
  <Card.Root
    class="w-full h-full border border-[#1F1F1F] shadow-[0px_4px_14px_2px_#0B8CE9] rounded-[15px] transition-all hover:scale-[1.02] cursor-pointer"
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

      <!-- More Options -->
      <DropdownMenu.Root bind:open={menuOpen}>
        <DropdownMenu.Trigger>
          <button class="absolute right-3 top-3" on:click|stopPropagation>
            <MoreVertical class="w-3 h-[13px] text-white/75" />
          </button>
        </DropdownMenu.Trigger>

        <CanisterOptionsMenu
          {canisterId}
          canisterName={statusInfo?.name || ""}
          on:canisterRenamed={handleCanisterRenamed}
          on:canisterDeleted={handleCanisterDeleted}
        />
      </DropdownMenu.Root>

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

          <!-- Memory -->
          <div class="space-y-2 border-b border-white/20 pb-2">
            <div class="text-[11px] leading-[14px] text-white">Memory</div>
            {#if statusInfo}
              <div
                class="relative h-[10px] bg-[#B6C8CE] rounded-full overflow-hidden"
              >
                <div
                  class="absolute left-0 top-0 h-full bg-[#0B8CE9] rounded-full"
                  style:width={`${(Number(statusInfo.memorySize) / Number(statusInfo.memoryAllocation)) * 100}%`}
                ></div>
              </div>
              <div class="text-[11px] leading-[14px] text-white">
                {formatGB(statusInfo.memorySize)} / {formatGB(
                  statusInfo.memoryAllocation,
                )} GB
              </div>
            {:else}
              <div>Loading memory...</div>
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
