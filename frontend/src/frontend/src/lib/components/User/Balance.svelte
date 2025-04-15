<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { authStore, type AuthStateAuthenticated } from '$lib/services/auth';
  import { BalanceService, type BalanceState } from '$lib/services/balance';
  import { writable, type Writable } from 'svelte/store';

  let balanceService: BalanceService | null = null;
  // Use a local store to hold the balance state from the service
  const balanceStore: Writable<BalanceState> = writable({
    balance: null,
    loading: false,
    error: null,
  });

  let unsubscribeAuth: (() => void) | null = null;
  let unsubscribeBalance: (() => void) | null = null;

  onMount(() => {
    unsubscribeAuth = authStore.subscribe(async (authState) => {
      // Clean up previous balance service subscription if auth state changes
      if (unsubscribeBalance) {
        unsubscribeBalance();
        unsubscribeBalance = null;
      }
      if (balanceService) {
        balanceService.reset(); // Reset state if auth changes
        balanceService = null;
      }

      if (authState.state === 'authenticated') {
        const authenticatedState = authState as AuthStateAuthenticated;
        balanceService = new BalanceService(authenticatedState.authClient);

        // Subscribe to the service's store
        unsubscribeBalance = balanceService.store.subscribe(state => {
          balanceStore.set(state);
        });

        // Fetch initial balance
        await balanceService.fetchBalance();

      } else {
        // Reset local store if not authenticated
        balanceStore.set({ balance: null, loading: false, error: null });
      }
    });
  });

  onDestroy(() => {
    if (unsubscribeAuth) {
      unsubscribeAuth();
    }
    if (unsubscribeBalance) {
      unsubscribeBalance();
    }
    // Optional: Explicitly reset service state on destroy if needed elsewhere
    // if (balanceService) {
    //   balanceService.reset();
    // }
  });

  // Helper function to format balance (e8s to ICP)
  function formatBalance(e8s: bigint | null): string {
    if (e8s === null) return 'N/A';
    const icp = Number(e8s) / 10**8;
    // Format to a reasonable number of decimal places
    return icp.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 });
  }
</script>

<div class="p-4 border rounded-md shadow-sm">
  <h2 class="text-lg font-semibold mb-2">ICP Balance</h2>
  {#if $balanceStore.loading}
    <p>Loading balance...</p>
  {:else if $balanceStore.error}
    <p class="text-red-500">Error: {$balanceStore.error}</p>
  {:else if $balanceStore.balance !== null}
    <p class="text-xl font-bold">{formatBalance($balanceStore.balance)} ICP</p>
     <!-- Refresh button -->
     <button
       class="mt-2 px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded disabled:opacity-50"
       disabled={$balanceStore.loading}
       on:click={() => balanceService?.fetchBalance(true)}
     >
       Refresh (Certified)
     </button>
  {:else if $authStore.state === 'authenticated'}
     <p>Could not load balance.</p>
      <!-- Retry button -->
     <button
       class="mt-2 px-3 py-1 text-sm bg-gray-500 hover:bg-gray-600 text-white rounded disabled:opacity-50"
       disabled={$balanceStore.loading}
       on:click={() => balanceService?.fetchBalance()}
     >
       Retry
     </button>
  {:else}
    <p>Please log in to view your balance.</p>
  {/if}
</div>
