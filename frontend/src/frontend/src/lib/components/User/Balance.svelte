<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { authStore, type AuthStateAuthenticated } from "$lib/services/auth";
  import { BalanceService, type BalanceState } from "$lib/services/balance";
  import { writable, type Writable } from "svelte/store";
  import { AccountIdentifier } from "@dfinity/ledger-icp";
  import { Principal } from "@dfinity/principal";
  import { userStore } from "$lib/services/user";

  let balanceService: BalanceService | null = null;
  const balanceStore: Writable<BalanceState> = writable({
    balance: null,
    loading: false,
    error: null,
  });

  let principalId: Principal | null = null;
  let accountId: string | null = null;
  let showCopyTooltip = false;

  let unsubscribeAuth: (() => void) | null = null;
  let unsubscribeBalance: (() => void) | null = null;

  onMount(() => {
    unsubscribeAuth = authStore.subscribe(async (authState) => {
      if (unsubscribeBalance) {
        unsubscribeBalance();
        unsubscribeBalance = null;
      }
      if (balanceService) {
        balanceService.reset();
        balanceService = null;
        principalId = null;
        accountId = null;
      }

      if (authState.state === "authenticated") {
        const authenticatedState = authState as AuthStateAuthenticated;
        principalId = authenticatedState.authClient
          .getIdentity()
          .getPrincipal();
        accountId = AccountIdentifier.fromPrincipal({
          principal: principalId,
        }).toHex();

        balanceService = new BalanceService(authenticatedState.authClient);

        unsubscribeBalance = balanceService.store.subscribe((state) => {
          balanceStore.set(state);
        });

        await balanceService.fetchBalance();
      } else {
        balanceStore.set({ balance: null, loading: false, error: null });
        principalId = null;
        accountId = null;
      }
    });
  });

  onDestroy(() => {
    if (unsubscribeAuth) unsubscribeAuth();
    if (unsubscribeBalance) unsubscribeBalance();
  });

  function formatBalance(e8s: bigint | null): string {
    if (e8s === null) return "";
    const icp = Number(e8s) / 10 ** 8;
    return icp.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 8,
    });
  }

  async function copyToClipboard(text: string) {
    await navigator.clipboard.writeText(text);
    showCopyTooltip = true;
    setTimeout(() => (showCopyTooltip = false), 2000);
  }

  function truncateAddress(address: string): string {
    if (!address) return "";
    return `${address.slice(0, 15)}...`;
  }
</script>

<div class="w-[286px] bg-[#1F1F1F] border border-[#0B8CE9] rounded-[21px] p-6">
  <h2 class="text-[17px] font-['Inder'] text-white mb-4">My Account</h2>

  {#if $userStore.state === "registered"}
    <div class="flex items-center justify-between mb-3">
      <p class="text-[15px] font-['Inder'] text-white">
        Name: {$userStore.username}
      </p>
      <button class="edit-button">
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M1 13L13 1M13 1H1M13 1V13"
            stroke="#0B8CE9"
            stroke-width="2"
          />
        </svg>
      </button>
    </div>
  {/if}

  <div class="mb-3">
    <p class="text-[15px] font-['Inder'] text-white">
      ICP Balance: {formatBalance($balanceStore.balance)}
    </p>
  </div>

  <div class="mb-4">
    <button class="text-[16px] font-['Inder'] text-white"> Add Funds </button>
  </div>

  <div class="currency-selector mb-4">
    <div
      class="border border-[#0B8CE9] rounded-[9px] p-3 flex justify-between items-center"
    >
      <span class="text-[16px] font-['Inder'] text-white">ICP</span>
      <svg
        width="10"
        height="6"
        viewBox="0 0 10 6"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path d="M1 1L5 5L9 1" stroke="white" stroke-width="2" />
      </svg>
    </div>
  </div>

  <div class="mt-4">
    <p class="text-[16px] font-['Inder'] text-white mb-2">
      Send ICP to your account:
    </p>
    <div class="flex items-center justify-between">
      <span class="text-[16px] font-['Inder'] text-[#0B8CE9]">
        {accountId ? truncateAddress(accountId) : "N/A"}
      </span>
      {#if accountId}
        <button
          class="copy-button relative"
          on:click={() => copyToClipboard(accountId)}
        >
          <div class="flex">
            <div
              class="w-[10px] h-[10px] border border-[#0B8CE9] rounded-[2px] bg-[#1F1F1F] absolute top-0 right-0"
            ></div>
            <div
              class="w-[10px] h-[10px] border border-[#0B8CE9] rounded-[2px] bg-[#1F1F1F] absolute top-1 right-1"
            ></div>
          </div>
          {#if showCopyTooltip}
            <span
              class="absolute -top-8 right-0 bg-white text-black text-xs px-2 py-1 rounded"
            >
              Copied!
            </span>
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .edit-button,
  .copy-button {
    @apply hover:opacity-80 transition-opacity;
  }
</style>
