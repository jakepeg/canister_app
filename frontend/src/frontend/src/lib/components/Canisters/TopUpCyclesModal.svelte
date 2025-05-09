<!-- src/lib/components/Canisters/TopUpCyclesModal.svelte -->
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import type { Principal } from "@dfinity/principal";
  import * as Dialog from "$lib/components/ui/dialog";
  import Input from "$lib/components/ui/input/input.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import {
    topUpCanisterWithCycles,
    DEFAULT_ICP_TRANSFER_FEE,
  } from "$lib/services/canisterManagement";
  import { authStore, type AuthStateAuthenticated } from "$lib/services/auth";
  import { BalanceService } from "$lib/services/balance";
  import { enumIs } from "$lib/shared/enums";

  type Props = {
    open: boolean;
    canisterId: Principal;
    canisterName: string;
    currentCyclesT: string; // Formatted string like "0.123 T"
  };
  let {
    open = $bindable(),
    canisterId,
    canisterName,
    currentCyclesT,
  }: Props = $props();

  const dispatch = createEventDispatcher<{
    close: void;
    topupSuccess: { message: string };
  }>();

  let icpAmountStr = $state("0.5"); // Default to 0.5 ICP
  let isLoading = $state(false);
  let errorMsg = $state<string | null>(null);
  let successMsg = $state<string | null>(null);

  let userIcpBalanceE8s = $state<bigint | null>(null);
  let userIcpBalanceLoading = $state(true);
  let balanceServiceInstance: BalanceService | null = $state(null); // Corrected: make it assignable
  let balanceUnsubscribe: (() => void) | undefined = undefined;

  onMount(() => {
    const auth = get(authStore);
    if (auth.state === "authenticated") {
      balanceServiceInstance = new BalanceService(auth.authClient);
      balanceUnsubscribe = balanceServiceInstance.store.subscribe((state) => {
        userIcpBalanceE8s = state.balance;
        userIcpBalanceLoading = state.loading;
        if (state.error && open) {
          // Only log error if modal is relevant/open
          console.warn(
            "Error fetching user balance for top-up modal:",
            state.error,
          );
        }
      });
      balanceServiceInstance.fetchBalance();
    } else {
      userIcpBalanceLoading = false;
    }
  });

  onDestroy(() => {
    if (balanceUnsubscribe) balanceUnsubscribe();
    balanceServiceInstance?.reset();
  });

  function formatIcp(e8s: bigint | null, withLabel = true): string {
    if (e8s === null) return "N/A";
    const label = withLabel ? " ICP" : "";
    return (Number(e8s) / 1e8).toFixed(4) + label;
  }

  async function handleSubmit() {
    isLoading = true;
    errorMsg = null;
    successMsg = null;

    const icpAmountNum = parseFloat(icpAmountStr);
    if (isNaN(icpAmountNum) || icpAmountNum <= 0) {
      errorMsg = "Please enter a valid positive ICP amount.";
      isLoading = false;
      return;
    }

    const icpAmountToConvertE8s = BigInt(Math.round(icpAmountNum * 1e8));

    if (
      userIcpBalanceE8s !== null &&
      icpAmountToConvertE8s + DEFAULT_ICP_TRANSFER_FEE > userIcpBalanceE8s
    ) {
      errorMsg = `Insufficient ICP balance. You need at least ${formatIcp(icpAmountToConvertE8s + DEFAULT_ICP_TRANSFER_FEE)}. Available: ${formatIcp(userIcpBalanceE8s)}`;
      isLoading = false;
      return;
    }

    if (icpAmountToConvertE8s <= 0n) {
      // This was checked by icpAmountNum > 0, but good to be explicit for bigint
      errorMsg = `ICP amount for cycles must be positive.`;
      isLoading = false;
      return;
    }

    const result = await topUpCanisterWithCycles(
      canisterId,
      icpAmountToConvertE8s,
    );

    if (enumIs(result, "ok")) {
      successMsg =
        result.message || "Top-up successful! Cycles will update shortly.";
      // Refresh user's balance as it has changed
      if (balanceServiceInstance) {
        await balanceServiceInstance.fetchBalance();
      }
      // Parent will dispatch topupSuccess which leads to CanisterCard status refresh
    } else {
      errorMsg = result.err;
    }
    isLoading = false;
    if (enumIs(result, "ok")) {
      // Only dispatch success if it was truly okay
      // Defer dispatch to allow UI to update with successMsg first
      setTimeout(() => {
        dispatch("topupSuccess", { message: successMsg! });
      }, 100);
    }
  }

  function handleOpenChange(newOpenState: boolean) {
    if (!newOpenState && open) {
      closeModalInternally();
    }
    // If opening, reset messages if not a success case
    if (newOpenState && !successMsg) {
      errorMsg = null;
    }
  }

  function closeModalInternally() {
    // This is called when dialog closes itself (e.g. Esc, overlay click)
    // We need to inform the parent via bind:open mechanism, which happens implicitly
    // and also call our dispatch if needed.
    if (!successMsg) {
      // If not closed after success message
      errorMsg = null;
    }
    dispatch("close"); // Always dispatch close so parent can react
  }

  function requestClose() {
    // Called by Cancel or Done button
    successMsg = null; // Reset success message on manual close
    errorMsg = null;
    // icpAmountStr = "0.5"; // Optionally reset input
    dispatch("close");
  }
</script>

<Dialog.Root bind:open onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] dark:bg-[#1F1F1F] bg-white p-6 shadow-lg rounded-[21px] font-inder dark:text-white text-gray-900"
    >
      <Dialog.Header>
        <Dialog.Title class="text-lg mb-1">Top Up Cycles</Dialog.Title>
        <Dialog.Description
          class="text-sm text-muted-foreground mb-2 dark:text-gray-400 text-gray-600"
        >
          For canister: {canisterName} ({canisterId
            .toText()
            .substring(0, 5)}...)
          <br />
          Current Cycles: {currentCyclesT}
        </Dialog.Description>
      </Dialog.Header>

      {#if successMsg && !isLoading}
        <div class="text-green-500 p-3 my-2 bg-green-500/10 rounded-md">
          {successMsg}
        </div>
        <div class="flex justify-end mt-4">
          <Button variant="outline" onclick={requestClose}>Done</Button>
        </div>
      {:else}
        <div class="space-y-3">
          {#if userIcpBalanceLoading}
            <p class="text-sm">Loading your ICP balance...</p>
          {:else if userIcpBalanceE8s !== null}
            <p class="text-sm">
              Your ICP Balance: {formatIcp(userIcpBalanceE8s)}
            </p>
          {:else}
            <p class="text-sm text-orange-500">Could not load ICP balance.</p>
          {/if}

          <div>
            <label for="icpAmount" class="text-sm font-medium"
              >ICP to convert to cycles:</label
            >
            <Input
              id="icpAmount"
              type="number"
              placeholder="e.g., 0.5"
              bind:value={icpAmountStr}
              min="0.0001"
              step="0.0001"
              class="bg-transparent border border-[#0B8CE9] rounded-[9px] mt-1"
              disabled={isLoading}
            />
            <p
              class="text-xs text-muted-foreground mt-1 dark:text-gray-400 text-gray-600"
            >
              A transfer fee of {formatIcp(DEFAULT_ICP_TRANSFER_FEE)} will be added.
              Total ICP deducted: {formatIcp(
                (parseFloat(icpAmountStr) > 0
                  ? BigInt(Math.round(parseFloat(icpAmountStr) * 1e8))
                  : 0n) + DEFAULT_ICP_TRANSFER_FEE,
                false,
              )}.
            </p>
          </div>

          {#if errorMsg}
            <p class="text-red-500 text-sm mt-2 p-2 bg-red-500/10 rounded-md">
              {errorMsg}
            </p>
          {/if}
        </div>
        <Dialog.Footer class="mt-6 flex justify-end gap-2">
          <Button variant="outline" onclick={requestClose} disabled={isLoading}
            >Cancel</Button
          >
          <Button variant="default" onclick={handleSubmit} disabled={isLoading}>
            {isLoading ? "Processing..." : "Top Up"}
          </Button>
        </Dialog.Footer>
      {/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
