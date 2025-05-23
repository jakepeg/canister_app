<script lang="ts">
  import { onMount, onDestroy } from "svelte"; // Import onDestroy
  import { afterNavigate } from "$app/navigation"; // <-- Import afterNavigate
  import { type ActorSubclass } from "@dfinity/agent"; // Import ActorSubclass
  import NotAuthenticated from "$lib/components/Home/NotAuthenticated.svelte";
  import { get } from "svelte/store"; // Import get
  import { authStore, type AuthStateAuthenticated } from "$lib/services/auth"; // Keep only this authStore import
  import CanisterList from "$lib/components/Canisters/CanisterList.svelte";
  import CreateCanisterModal from "$lib/components/Canisters/CreateCanisterModal.svelte";
  // Corrected import path again (relative to src/frontend/src/routes)
  import type {
    CanisterInfo as BackendCanisterInfo,
    _SERVICE as BackendService,
  } from "../../../declarations/backend/backend.did";

  import { BalanceService, type BalanceState } from "$lib/services/balance";

  // Local type alias for the component prop, expecting id as string
  type ComponentCanisterInfo = {
    id: string; // Expecting string ID for the component
    name: string;
    // iconUrl?: string; // Add if needed
  };

  // Svelte 5 Runes for state
  let canisters = $state<ComponentCanisterInfo[]>([]);
  let isLoadingCanisters = $state(true);
  let isModalOpen = $state(false);
  let fetchError = $state("");

  // --- Balance State ---
  let pageBalanceService: BalanceService | null = $state(null);
  let icpBalanceE8s: bigint | null = $state(null);
  let icpBalanceLoading = $state(true); // Start as true until first fetch attempt
  let icpBalanceError: string | null = $state(null);

  // Derived state for sufficient funds (1 ICP = 100,000,000 e8s)
  let hasSufficientFunds = $derived(
    icpBalanceE8s !== null && icpBalanceE8s >= 100_000_000n,
  );
  // --- End Balance State ---

  let authUnsubscribe: (() => void) | undefined = undefined;
  let balanceUnsubscribe: (() => void) | undefined = undefined; // For balance store subscription

  // Get the main backend canister ID (ensure this env var is set correctly)
  const MAIN_BACKEND_CANISTER_ID = import.meta.env.VITE_BACKEND_CANISTER_ID;
  if (!MAIN_BACKEND_CANISTER_ID) {
    console.error(
      "VITE_BACKEND_CANISTER_ID is not set in environment variables!",
    );
    fetchError = "Application configuration error: Missing backend ID.";
    isLoadingCanisters = false; // Stop loading i0f config is broken
  }
  console.log(
    "+page.svelte: MAIN_BACKEND_CANISTER_ID:",
    MAIN_BACKEND_CANISTER_ID,
  );

  async function fetchCanisters() {
    // Only proceed if authenticated
    const authState = get(authStore);
    if (authState.state !== "authenticated") {
      console.log("+page.svelte: Not authenticated, cannot fetch canisters.");
      canisters = [];
      isLoadingCanisters = false; // Not loading if not auth'd
      fetchError = ""; // Clear previous errors
      return;
    }

    // If already fetching, don't start another one (basic guard)
    // A more robust solution might use a status like 'fetching'
    // if (isLoadingCanisters && fetchError === "") return; // Guard against re-entry if already fetching

    isLoadingCanisters = true;
    fetchError = "";
    console.log("+page.svelte: Fetching canisters...");

    const actor = authState.actor as ActorSubclass<BackendService>;
    if (!actor) {
      console.error("+page.svelte: Backend actor not available for fetching.");
      fetchError = "Backend actor not available.";
      canisters = []; // Clear canisters on error
      isLoadingCanisters = false;
      return;
    }

    try {
      const result = await actor.get_user_canisters();
      if ("Ok" in result) {
        canisters = result.Ok.map((backendInfo: BackendCanisterInfo) => ({
          id: backendInfo.id.toText(),
          name: backendInfo.name,
        }));
        console.log("+page.svelte: Canisters fetched:", canisters.length);
      } else if ("NotAuthenticated" in result) {
        console.warn(
          "+page.svelte: Backend reported user not authenticated during fetch.",
        );
        fetchError = "Not authenticated according to backend.";
        canisters = [];
      } else {
        const errorKey = Object.keys(result)[0];
        const errorValue = (result as any)[errorKey];
        console.error(
          `+page.svelte: Error from get_user_canisters: ${errorKey}`,
          errorValue,
        );
        fetchError = `Error from backend: ${errorKey}`;
        canisters = [];
      }
    } catch (err: any) {
      console.error("+page.svelte: Error fetching canisters:", err);
      fetchError = `Error fetching canisters: ${err.message || "Unknown error"}`;
      canisters = [];
    } finally {
      isLoadingCanisters = false; // Always set to false after attempt
      console.log(
        "+page.svelte: fetchCanisters finished. isLoadingCanisters:",
        isLoadingCanisters,
      );
    }
  }

  // Function to refresh list after creation - just re-fetches
  function handleCanisterCreated() {
    console.log("Canister created event received, refreshing list...");
    fetchCanisters(); // Re-fetch from backend
  }

  function openCreateCanisterModal() {
    isModalOpen = true;
  }

  function handleCreateModalOpenChange(newOpenState: boolean) {
    console.log(
      `+page.svelte: handleCreateModalOpenChange called. Requested new state: ${newOpenState}. Current isModalOpen: ${isModalOpen}`,
    );
    isModalOpen = newOpenState;
    if (!newOpenState) {
      console.log(
        "+page.svelte: CreateCanisterModal is NOW CLOSED by onOpenChange. isModalOpen final value:",
        isModalOpen,
      );
    } else {
      console.log(
        "+page.svelte: CreateCanisterModal is NOW OPENED by onOpenChange. isModalOpen final value:",
        isModalOpen,
      );
    }
  }

  onMount(() => {
    console.log("+page.svelte: Mounted.");

    // Subscribe to authStore changes
    authUnsubscribe = authStore.subscribe(async (currentAuth) => {
      console.log(
        "+page.svelte: Auth store changed:",
        currentAuth.state,
        "isLoading:",
        isLoadingCanisters,
      );

      // Reset balance state and unsubscribe on auth change
      if (balanceUnsubscribe) {
        balanceUnsubscribe();
        balanceUnsubscribe = undefined;
      }
      if (pageBalanceService) {
        pageBalanceService.reset(); // Assuming BalanceService has a reset method
        pageBalanceService = null;
      }
      icpBalanceE8s = null;
      icpBalanceLoading = true; // Set to true until fetch attempt
      icpBalanceError = null;

      if (currentAuth.state === "authenticated") {
        // If authenticated, and we are not already successfully loaded
        // (isLoadingCanisters is true, or no canisters and no error)
        // then attempt a fetch. afterNavigate is the primary fetch trigger on page load.
        if (isLoadingCanisters || (canisters.length === 0 && !fetchError)) {
          console.log(
            "+page.svelte: Auth store became authenticated, triggering fetch (if needed).",
          );

          fetchCanisters(); // Call fetch if conditions met
        } else {
          console.log(
            "+page.svelte: Auth store authenticated, but canisters seem loaded or error occurred. No fetch from subscribe.",
          );
        }

        // Initialize and fetch balance
        console.log(
          "+page.svelte: Authenticated, initializing BalanceService.",
        );
        pageBalanceService = new BalanceService(currentAuth.authClient);
        balanceUnsubscribe = pageBalanceService.store.subscribe((state) => {
          icpBalanceE8s = state.balance;
          icpBalanceLoading = state.loading;
          icpBalanceError = state.error;
          console.log("+page.svelte: Balance state updated:", state);
        });
        try {
          await pageBalanceService.fetchBalance();
        } catch (err) {
          console.error(
            "+page.svelte: Error fetching balance on auth change:",
            err,
          );
          // error is already set in icpBalanceError via store subscription
        }
      } else if (currentAuth.state === "unauthenticated") {
        console.log(
          "+page.svelte: Auth store is not-authenticated. Clearing canisters, stopping load.",
        );
        canisters = [];
        isLoadingCanisters = false;
        fetchError = "";
      } else if (currentAuth.state === "uninitialized") {
        console.log(
          "+page.svelte: Auth store is uninitialized. Setting loading true.",
        );
        isLoadingCanisters = true;
        canisters = [];
        fetchError = "";
      }
    });

    // Set up afterNavigate hook *once* on mount
    afterNavigate(() => {
      console.log("+page.svelte: afterNavigate triggered.");
      const currentAuthStateOnNav = get(authStore); // Get fresh auth state on navigate
      console.log(
        "+page.svelte: Auth state in afterNavigate:",
        currentAuthStateOnNav.state,
      );

      if (currentAuthStateOnNav.state === "authenticated") {
        console.log(
          "+page.svelte: Authenticated in afterNavigate, calling fetchCanisters.",
        );
        fetchCanisters();
      } else if (currentAuthStateOnNav.state === "unauthenticated") {
        console.log(
          "+page.svelte: Not authenticated in afterNavigate. Clearing canisters, stopping load.",
        );
        canisters = [];
        isLoadingCanisters = false;
        fetchError = "";
      } else {
        // uninitialized
        console.log(
          "+page.svelte: Auth uninitialized in afterNavigate. Ensuring loading state.",
        );
        isLoadingCanisters = true;
        canisters = [];
        fetchError = "";
      }
    });

    // Initial check based on current auth state AT MOUNT TIME
    // This helps set the initial isLoadingCanisters correctly before afterNavigate or subscribe fully kick in.
    const initialAuthStateOnMount = get(authStore);
    console.log(
      "+page.svelte: Initial auth state on mount (direct check):",
      initialAuthStateOnMount.state,
    );
    if (initialAuthStateOnMount.state === "uninitialized") {
      isLoadingCanisters = true;
    } else if (initialAuthStateOnMount.state === "unauthenticated") {
      isLoadingCanisters = false;
      canisters = [];
      fetchError = "";
    } else if (initialAuthStateOnMount.state === "authenticated") {
      // If authenticated on mount, afterNavigate will trigger the fetch.
      // We might still be in isLoadingCanisters = true from its declaration, which is fine.
      // Or if canisters were somehow loaded by a parent (not in this case for a page),
      // we could potentially skip setting isLoadingCanisters = true here.
      // For now, let afterNavigate handle the fetch.
      console.log(
        "+page.svelte: Authenticated on mount (direct check). afterNavigate will fetch.",
      );
    }
  }); // --- End of onMount ---

  onDestroy(() => {
    if (authUnsubscribe) {
      authUnsubscribe();
      console.log("+page.svelte: Unsubscribed from authStore.");
    }
    if (balanceUnsubscribe) {
      balanceUnsubscribe();
      console.log("+page.svelte: Unsubscribed from balanceStore.");
    }
    if (pageBalanceService) {
      pageBalanceService.reset(); // Clean up the service instance
    }
    console.log("+page.svelte: Destroyed."); // Add this to see when/if it's destroyed
  });
</script>

<section class="w-full">
  {#if isLoadingCanisters && $authStore.state !== "unauthenticated"}
    <div class="flex justify-center items-center h-screen">
      <h1 class="text-xl text-white">Loading...</h1>
    </div>
  {:else if $authStore.state === "authenticated"}
    {#if fetchError}
      <div class="text-red-500 p-4 text-center">Error: {fetchError}</div>
    {/if}
    <CanisterList
      {canisters}
      onOpenCreateModal={openCreateCanisterModal}
      onRefreshCanisters={fetchCanisters}
      {hasSufficientFunds}
      balanceLoading={icpBalanceLoading}
    />

    <!-- Render Create Canister Modal -->
    <CreateCanisterModal
      open={isModalOpen}
      onOpenChange={handleCreateModalOpenChange}
      onCanisterCreated={handleCanisterCreated}
    />
  {:else}
    <!-- Render Not Authenticated Component -->
    <NotAuthenticated />
  {/if}
</section>
