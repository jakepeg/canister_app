<script lang="ts">
	import { onMount } from 'svelte';
	import { type ActorSubclass } from '@dfinity/agent'; // Import ActorSubclass
	import NotAuthenticated from '$lib/components/Home/NotAuthenticated.svelte';
	import { get } from 'svelte/store'; // Import get
	import { authStore, type AuthStateAuthenticated } from '$lib/services/auth'; // Keep only this authStore import
	import CanisterList from '$lib/components/Canisters/CanisterList.svelte';
	import CreateCanisterModal from '$lib/components/Canisters/CreateCanisterModal.svelte';
	// Corrected import path again (relative to src/frontend/src/routes)
	import type { CanisterInfo as BackendCanisterInfo, _SERVICE as BackendService } from '../../../declarations/backend/backend.did';

	// Local type alias for the component prop, expecting id as string
	type ComponentCanisterInfo = {
		id: string; // Expecting string ID for the component
		name: string;
		// iconUrl?: string; // Add if needed
	};

	let canisters: ComponentCanisterInfo[] = []; // Use the local type alias
	let isLoadingCanisters = true;
	let isModalOpen = false;
	let fetchError = '';

	// Function to fetch canisters from the backend
	async function fetchCanisters() {
		isLoadingCanisters = true;
		fetchError = '';
		console.log('Fetching canisters from backend...');

		const authState = get(authStore);
		if (authState.state !== 'authenticated') {
			console.log('Not authenticated, skipping fetch.');
			isLoadingCanisters = false;
			canisters = []; // Clear canisters if not authenticated
			return;
		}

		// Cast actor to the imported BackendService type
		const actor = (authState as AuthStateAuthenticated).actor as ActorSubclass<BackendService>;
		if (!actor) {
			console.error('Backend actor not available.');
			fetchError = 'Backend actor not available.';
			isLoadingCanisters = false;
			return;
		}

		try {
			const result = await actor.get_user_canisters();
			if ('Ok' in result) {
				// Map backend CanisterInfo (with Principal id) to ComponentCanisterInfo (with string id)
				canisters = result.Ok.map((backendInfo: BackendCanisterInfo) => ({
					id: backendInfo.id.toText(), // Convert Principal to string
					name: backendInfo.name
					// Map other fields if necessary
				}));
				console.log('Canisters fetched:', canisters);
			} else if ('NotAuthenticated' in result) {
				console.warn('Backend reported user not authenticated.');
				fetchError = 'Not authenticated according to backend.';
				canisters = [];
			} else {
				console.error('Unknown response from get_user_canisters:', result);
				fetchError = 'Unknown error fetching canisters.';
				canisters = [];
			}
		} catch (err: any) {
			console.error('Error fetching canisters:', err);
			fetchError = `Error fetching canisters: ${err.message || 'Unknown error'}`;
			canisters = [];
		} finally {
			isLoadingCanisters = false;
		}
	}

	// Function to refresh list after creation - just re-fetches
	function handleCanisterCreated() {
		console.log('Canister created event received, refreshing list...');
		fetchCanisters(); // Re-fetch from backend
	}

	function openModal() {
		isModalOpen = true;
	}

	function closeModal() {
		isModalOpen = false;
	}

	onMount(() => {
		if ($authStore.state === 'authenticated') {
			fetchCanisters();
		}
	});

	// Re-fetch if auth state changes to authenticated after mount
	$: if ($authStore.state === 'authenticated' && canisters.length === 0 && !isLoadingCanisters) {
		fetchCanisters();
	}
</script>

<section class="w-full">
	{#if $authStore.state === 'uninitialized' || ($authStore.state === 'authenticated' && isLoadingCanisters)}
		<!-- Unified Loading State -->
		<div class="flex justify-center items-center h-screen">
			<h1 class="text-xl text-white">Loading...</h1>
			<!-- Optional: Add a spinner here -->
		</div>
	{:else if $authStore.state === 'authenticated'}
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
