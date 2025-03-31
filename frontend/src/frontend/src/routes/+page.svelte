<script lang="ts">
	import { onMount } from 'svelte';
	import NotAuthenticated from '$lib/components/Home/NotAuthenticated.svelte';
	import { authStore } from '$lib/services/auth';
	import CanisterList from '$lib/components/Canisters/CanisterList.svelte';
	import CreateCanisterModal from '$lib/components/Canisters/CreateCanisterModal.svelte';

	// Define mock CanisterInfo type matching CanisterList
	type CanisterInfo = {
		id: string;
		name: string;
		iconUrl?: string;
	};

	let canisters: CanisterInfo[] = [];
	let isLoadingCanisters = true;
	let isModalOpen = false;

	// Mock function to fetch canisters
	async function fetchCanisters() {
		isLoadingCanisters = true;
		console.log('Fetching canisters...');
		// Simulate network delay
		await new Promise((resolve) => setTimeout(resolve, 1000));
		// Mock data - replace with actual backend call
		canisters = [
			{ id: 'canister-1', name: 'My First Canister' },
			// { id: 'canister-2', name: 'Project Data' },
			// { id: 'canister-3', name: 'Shared Files' }
		];
		// To test empty state, keep the array empty initially
		console.log('Canisters fetched:', canisters);
		isLoadingCanisters = false;
	}

	// Mock function to refresh list after creation
	function handleCanisterCreated() {
		console.log('Canister created event received, refreshing list...');
		// In a real app, re-fetch or add the new canister to the list
		// For mock, let's add a new one
		const newId = `canister-${Date.now()}`;
		canisters = [...canisters, { id: newId, name: `New Canister ${canisters.length + 1}` }];
		fetchCanisters(); // Or just update the local state: canisters = newCanisters;
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
