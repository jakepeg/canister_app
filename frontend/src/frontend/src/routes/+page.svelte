<script lang="ts">
  import { onMount, onDestroy } from "svelte"; // Import onDestroy
  import { afterNavigate } from "$app/navigation"; // <-- Import afterNavigate
  import { type ActorSubclass } from "@dfinity/agent"; // Import ActorSubclass
  import NotAuthenticated from "$lib/components/Home/NotAuthenticated.svelte";
  import { get } from "svelte/store"; // Import get
  import {
    authStore,
    authService,
    type AuthStateAuthenticated,
  } from "$lib/services/auth"; // Keep only this authStore import
  import CanisterList from "$lib/components/Canisters/CanisterList.svelte";
  import CreateCanisterModal from "$lib/components/Canisters/CreateCanisterModal.svelte";
  // Corrected import path again (relative to src/frontend/src/routes)
  import type {
    CanisterInfo as BackendCanisterInfo,
    _SERVICE as BackendService,
  } from "../../../declarations/backend/backend.did";

  // Local type alias for the component prop, expecting id as string
  type ComponentCanisterInfo = {
    id: string; // Expecting string ID for the component
    name: string;
    // iconUrl?: string; // Add if needed
  };

  let canisters: ComponentCanisterInfo[] = []; // Use the local type alias
  let isLoadingCanisters = true;
  let isModalOpen = false;
  let fetchError = "";
  let unsubscribeAuth: () => void; // To store the unsubscribe function
  const canisterId = import.meta.env.VITE_FRONTEND_CANISTER_ID;
  console.log("canisterId", canisterId);

  // $: {
  //   if (canisterId) {
  //     authService.setCurrentCanister(canisterId);
  //   }
  // }

  // Function to fetch canisters from the backend
  async function fetchCanisters() {
    // Ensure loading state is set correctly at the start
    isLoadingCanisters = true;
    fetchError = "";
    console.log("Fetching canisters from backend...");

    const authState = get(authStore);
    if (authState.state !== "authenticated") {
      console.log("Not authenticated, skipping fetch.");
      canisters = []; // Clear canisters
      isLoadingCanisters = false; // Stop loading
      return;
    }

    const actor = authState.actor as ActorSubclass<BackendService>;
    if (!actor) {
      console.error("Backend actor not available.");
      fetchError = "Backend actor not available.";
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
        console.log("Canisters fetched:", canisters);
      } else if ("NotAuthenticated" in result) {
        console.warn("Backend reported user not authenticated.");
        fetchError = "Not authenticated according to backend.";
        canisters = [];
      } else {
        console.error("Unknown response from get_user_canisters:", result);
        fetchError = "Unknown error fetching canisters.";
        canisters = [];
      }
    } catch (err: any) {
      console.error("Error fetching canisters:", err);
      fetchError = `Error fetching canisters: ${err.message || "Unknown error"}`;
      canisters = [];
    } finally {
      // Ensure loading state is turned off
      isLoadingCanisters = false;
    }
  }

  // Function to refresh list after creation - just re-fetches
  function handleCanisterCreated() {
    console.log("Canister created event received, refreshing list...");
    fetchCanisters(); // Re-fetch from backend
  }

  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }

  onMount(() => {
    console.log("Main page mounted");
    unsubscribeAuth = authStore.subscribe((authState) => {
      // Fetch when authentication state changes *to* authenticated
      if (authState.state === "authenticated") {
        console.log(
          "Auth state changed to authenticated, fetching canisters...",
        );
        // Only fetch if canisters haven't been loaded yet *by this subscription trigger*
        // Let onMount / afterNavigate handle the initial/navigation load
        // This avoids double-fetching if auth resolves quickly after mount
        if (isLoadingCanisters || (canisters.length === 0 && !fetchError)) {
          fetchCanisters();
        }
      } else {
        // Reset state if user logs out or state changes otherwise
        canisters = [];
        isLoadingCanisters = true; // Set to true until we know auth state
        fetchError = "";
      }
    });

    // Initial check needed in case already authenticated when mounting
    // Let afterNavigate handle this for consistency, remove fetch from here.
    // const initialAuthState = get(authStore);
    // if (initialAuthState.state === 'authenticated') {
    //  console.log('Authenticated on mount, fetching canisters...');
    //  fetchCanisters();
    // } else {
    //  // If not authenticated on mount, set loading to false
    //  // The subscription will handle fetching if auth state changes later
    //  isLoadingCanisters = false;
    // }
  });

  onDestroy(() => {
    if (unsubscribeAuth) {
      unsubscribeAuth(); // Clean up the subscription
    }
  });

  // Removed the problematic reactive block that caused infinite loops
</script>

<section class="w-full">
  {#if $authStore.state === "uninitialized" || ($authStore.state === "authenticated" && isLoadingCanisters)}
    <!-- Unified Loading State -->
    <div class="flex justify-center items-center h-screen">
      <h1 class="text-xl text-white">Loading...</h1>
      <!-- Optional: Add a spinner here -->
    </div>
  {:else if $authStore.state === "authenticated"}
    <!-- Render Canister List -->
    <CanisterList {canisters} on:openCreateModal={openModal} />

    <!-- Render Create Canister Modal -->
    <CreateCanisterModal
      bind:open={isModalOpen}
      on:close={closeModal}
      on:canisterCreated={handleCanisterCreated}
    />
  {:else}
    <!-- Render Not Authenticated Component -->
    <NotAuthenticated />
  {/if}
</section>
