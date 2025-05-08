<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Label } from "$lib/components/ui/label";
  import { X } from "lucide-svelte";
  import { createAndRegisterCanister } from "$lib/services/canisterManagement";
  import { onMount, onDestroy } from "svelte";

  type Props = {
    open: boolean;
    onOpenChange: (isOpen: boolean) => void;
    onCanisterCreated?: () => void;
  };
  let { open, onOpenChange, onCanisterCreated }: Props = $props();

  let canisterName = $state("");
  let canisterSize = $state(10);
  let isLoading = $state(false);
  let error = $state("");

  function attemptCloseModal() {
    console.log(
      "CreateCanisterModal: attemptCloseModal START. isLoading:",
      isLoading,
    );
    if (isLoading) {
      console.log(
        "CreateCanisterModal: attemptCloseModal - bailing because isLoading is true.",
      );
      return;
    }
    canisterName = "";
    canisterSize = 10;
    error = "";
    console.log(
      "CreateCanisterModal: attemptCloseModal - Calling onOpenChange(false)",
    );
    onOpenChange(false);
    console.log("CreateCanisterModal: attemptCloseModal END.");
  }

  async function handleCreateCanister() {
    if (!canisterName.trim()) {
      error = "Canister name cannot be empty.";
      return;
    }
    if (canisterSize <= 0) {
      error = "Canister size must be greater than 0 GB.";
      return;
    }
    isLoading = true;
    error = "";

    try {
      console.log(
        `CreateCanisterModal: Attempting to create: ${canisterName}, size: ${canisterSize}GB`,
      );
      const result = await createAndRegisterCanister(
        canisterName.trim(),
        canisterSize,
      );

      if ("ok" in result) {
        console.log(
          `CreateCanisterModal: Canister ${result.ok.toText()} created and registered!`,
        );

        // Google Analytics event tracking
        if (typeof gtag === "function") {
          gtag("event", "canister_creation", {
            event_category: "Canister",
            event_label: canisterName,
            value: canisterSize,
          });
          console.log("Google Analytics: Canister creation event tracked.");
        }

        // --- FIX: Set isLoading false BEFORE trying to close ---
        isLoading = false;
        console.log(
          "CreateCanisterModal: Set isLoading = false after success.",
        );
        // ------------------------------------------------------

        if (onCanisterCreated) {
          console.log(
            "CreateCanisterModal: Calling onCanisterCreated callback.",
          );
          onCanisterCreated();
        }
        console.log(
          "CreateCanisterModal: BEFORE calling attemptCloseModal after creation.",
        );
        attemptCloseModal();
        console.log(
          "CreateCanisterModal: AFTER calling attemptCloseModal after creation.",
        );
      } else {
        console.error(
          "CreateCanisterModal: Failed to create/register canister:",
          result.err,
        );
        error = `Failed: ${result.err}`;
      }
    } catch (err: any) {
      console.error(
        "CreateCanisterModal: Unexpected error during canister creation:",
        err,
      );
      error = `An unexpected error occurred: ${err.message || "Unknown error"}`;
    } finally {
      isLoading = false;
      console.log(
        "CreateCanisterModal: handleCreateCanister finally block. isLoading:",
        isLoading,
      );
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      attemptCloseModal();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
  });
  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<Dialog.Root
  {open}
  onOpenChange={(newOpenState) => {
    console.log(
      "CreateCanisterModal: Dialog.Root onOpenChange called with:",
      newOpenState,
    );
    onOpenChange(newOpenState);
    if (!newOpenState && !isLoading) {
      canisterName = "";
      canisterSize = 10;
      error = "";
      console.log(
        "CreateCanisterModal: Dialog.Root onOpenChange - form reset because closing.",
      );
    }
  }}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 " />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] p-6 shadow-lg duration-200 rounded-[21px] font-inder"
      aria-describedby="create-canister-description"
    >
      <Dialog.Header class="flex justify-between items-center">
        <Dialog.Title class="text-lg font-inder"
          >Create New Canister</Dialog.Title
        >
      </Dialog.Header>
      <Dialog.Description id="create-canister-description" class="sr-only">
        Modal to create a new canister by providing a name.
      </Dialog.Description>
      <div class="space-y-4 mt-4">
        <div>
          <label for="canisterName" class="block text-sm font-inder mb-1"
            >Canister Name</label
          >
          <Input
            id="canisterName"
            bind:value={canisterName}
            placeholder="Enter canister name"
            class="bg-transparent border border-[#0B8CE9] rounded-[9px] placeholder:font-inder placeholder:text-base focus:ring-1 focus:ring-[#0B8CE9]"
            disabled={isLoading}
          />
        </div>

        <!-- Size Input -->
        <!-- <div>
          <Label class="block text-sm font-inder mb-1">Size (GB)</Label>
          <Input
            type="number"
            bind:value={canisterSize}
            min="1"
            placeholder="Enter size in GB"
            class="bg-transparent border border-[#0B8CE9] rounded-[9px] placeholder:font-inder placeholder:text-base focus:ring-1 focus:ring-[#0B8CE9]"
            disabled={isLoading}
          />
        </div>
        <div>
          <Label class="block text-base font-inder mb-1">Setup Cost:</Label>
          <div class="font-inder text-base">
            TBD
          </div>
        </div>
      </div> -->

        <!-- Error Message -->
        {#if error}
          <p class="text-red-500 text-sm mt-2">{error}</p>
        {/if}

        <!-- Action Button -->
        <Dialog.Footer class="mt-6">
          <Button
            class="w-full font-inder text-base borderrounded-[22px]"
            variant="outline"
            onclick={handleCreateCanister}
            disabled={isLoading}
          >
            {#if isLoading}
              Creating...
            {:else}
              Create Canister
            {/if}
          </Button>
        </Dialog.Footer>
      </div></Dialog.Content
    >
  </Dialog.Portal>
</Dialog.Root>

<style>
  /* Ensure Inder font is loaded if not globally available */
  /* @import url('https://fonts.googleapis.com/css2?family=Inder&display=swap'); */
  .font-inder {
    font-family: "Inder", sans-serif;
  }
</style>
