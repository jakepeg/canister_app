<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    renameCanister,
    deleteCanister,
  } from "$lib/services/canisterManagement";
  import type { Principal } from "@dfinity/principal";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";

  type Props = {
    canisterId: Principal;
    canisterName: string;
    onCanisterRenamed?: () => void; // Callback prop
    onCanisterDeleted?: () => void; // Callback prop
  };
  let {
    canisterId,
    canisterName,
    onCanisterRenamed,
    onCanisterDeleted,
  }: Props = $props();

  let renameDialogOpen = false;
  let deleteDialogOpen = false;
  let newCanisterName = "";
  let isLoading = false;
  let error = "";

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
