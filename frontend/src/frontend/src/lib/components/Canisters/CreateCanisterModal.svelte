<!-- frontend/src/frontend/src/lib/components/Canisters/CreateCanisterModal.svelte -->
<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Label } from "$lib/components/ui/label";
  import { X } from "lucide-svelte";
  import { createAndRegisterCanister } from "$lib/services/canisterManagement"; // Import the service function
  import { onMount, onDestroy } from "svelte"; // For keydown listener

  // Svelte 5 Props
  type Props = {
    open: boolean;
    onOpenChange: (isOpen: boolean) => void; // Callback to notify parent about open state changes
    onCanisterCreated?: () => void; // Callback when canister is successfully created
  };
  let { open, onOpenChange, onCanisterCreated }: Props = $props();

  // Svelte 5 State Runes
  let canisterName = $state("");
  let canisterSize = $state(10); // Default size in GB
  let isLoading = $state(false);
  let error = $state("");

  function attemptCloseModal() {
    console.log(
      "CreateCanisterModal: attemptCloseModal START. isLoading:",
      isLoading,
    ); // <<< ADD LOG
    if (isLoading) {
      console.log(
        "CreateCanisterModal: attemptCloseModal - bailing because isLoading is true.",
      ); // <<< ADD LOG
      return;
    }
    canisterName = "";
    canisterSize = 10;
    error = "";
    console.log(
      "CreateCanisterModal: attemptCloseModal - Calling onOpenChange(false)",
    ); // <<< ADD LOG
    onOpenChange(false);
    console.log("CreateCanisterModal: attemptCloseModal END."); // <<< ADD LOG
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

        // --- FIX: Set isLoading false BEFORE trying to close ---
        isLoading = false;
        console.log(
          "CreateCanisterModal: Set isLoading = false after success.",
        );
        // ------------------------------------------------------

        if (onCanisterCreated) {
          console.log(
            "CreateCanisterModal: Calling onCanisterCreated callback.",
          ); // <<< ADD LOG
          onCanisterCreated();
        }
        console.log(
          "CreateCanisterModal: BEFORE calling attemptCloseModal after creation.",
        ); // <<< ADD LOG
        attemptCloseModal(); // Close the modal on success
        console.log(
          "CreateCanisterModal: AFTER calling attemptCloseModal after creation.",
        ); // <<< ADD LOG
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
      ); // <<< ADD LOG
    }
  }

  // Handle Escape key press to close modal
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      // Only act if modal is open
      attemptCloseModal();
    }
  }

  // Svelte 5: For `bind:value` on custom Input component, if it's not Svelte 5 compatible,
  // you'd need it to expose `value` prop and an `onInput` (or similar) event.
  // Assuming your Input component is Svelte 5 compatible and works with $state directly or
  // you modify it to accept value and emit changes.
  // If it's a simple wrapper, then:
  // <input value={canisterName} oninput={(e) => canisterName = e.target.value} ... />

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
    ); // <<< ADD LOG
    onOpenChange(newOpenState);
    if (!newOpenState && !isLoading) {
      // Only reset form if not loading
      canisterName = "";
      canisterSize = 10;
      error = "";
      console.log(
        "CreateCanisterModal: Dialog.Root onOpenChange - form reset because closing.",
      ); // <<< ADD LOG
    }
  }}
>
  <Dialog.Portal>
    <!-- Backdrop: Based on Rectangle 107 fill_RMREMI -->
    <Dialog.Overlay class="fixed inset-0 z-50 " />
    <!-- Modal Container: Based on 289:141 -->
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] p-6 shadow-lg duration-200 rounded-[21px] font-inder"
      aria-describedby="create-canister-description"
    >
      <!-- Header -->
      <Dialog.Header class="flex justify-between items-center">
        <!-- Title: Style: style_GUBF0I - Inder, 17px, White -->
        <Dialog.Title class="text-lg font-inder"
          >Create New Canister</Dialog.Title
        >
        <!-- Close Button: Style: style_GUBF0I - Inder, 17px, White -->
        <!-- Reverted to simple button with typed on:click -->
        <!-- <button
					on:click={(e: MouseEvent) => closeModal()}
					class="p-1 rounded-full hover:bg-white/10 transition-colors"
					aria-label="Close"
				>
					<X class="h-5 w-5" />
				</button> -->
      </Dialog.Header>

      <!-- Description for Accessibility -->
      <Dialog.Description id="create-canister-description" class="sr-only">
        Modal to create a new canister by providing a name.
      </Dialog.Description>

      <!-- Form Fields -->
      <div class="space-y-4 mt-4">
        <!-- Canister Name Input -->
        <div>
          <!-- Label: Style: style_4O2OYN - Inder, 15px, White -->
          <label for="canisterName" class="block text-sm font-inder mb-1"
            >Canister Name</label
          >
          <!-- Input: Based on Rectangle 93 (289:150) -->
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
          <!-- Button: Based on Rectangle 95 (289:154) -->
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
