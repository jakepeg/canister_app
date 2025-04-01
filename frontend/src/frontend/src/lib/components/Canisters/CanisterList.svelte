<!-- frontend/src/frontend/src/lib/components/Canisters/CanisterList.svelte -->
<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { goto } from '$app/navigation';
	import Button from '$lib/components/ui/button/button.svelte'; // Corrected path
	import * as Card from '$lib/components/ui/card'; // Corrected path

	// Define a mock CanisterInfo type for now
	type CanisterInfo = {
		id: string;
		name: string;
		iconUrl?: string; // Optional icon URL
	};

	export let canisters: CanisterInfo[] = [];

	const dispatch = createEventDispatcher();

	function openCreateModal() {
		dispatch('openCreateModal');
	}

	function navigateToCanisterFiles(canisterId: string) {
		// TODO: Confirm navigation path is correct
		goto(`/canister/${canisterId}/files`);
	}

	// Mock data for demonstration if needed (can be removed if parent passes data)
	// $: if (!canisters || canisters.length === 0) {
	//  canisters = [
	//      { id: '1', name: 'My First Canister' },
	//      { id: '2', name: 'Project Alpha' },
	//      { id: '3', name: 'Test Environment' }
	//  ];
	// }
</script>

<div class="container mx-auto px-4 py-8 ">
	<div class="flex justify-between items-center mb-6">
		<!-- Header: Style: style_ESKRTZ - Inder, 20px, White -->
		<h1 class="font-inder text-xl">My Canisters</h1>
		<!-- New Canister Button: Style: style_GUBF0I - Inder, 17px, White, white stroke, 6px border-radius -->
		<Button
			
			onclick={openCreateModal}
		>
			New Canister
		</Button>
	</div>

	{#if canisters.length > 0}
		<!-- Canister Grid: Based on Figma 288:76 -->
		<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
			{#each canisters as canister (canister.id)}
				<!-- Wrap Card.Root in a div to handle standard keydown event -->
				<div
					role="button"
					tabindex="0"
					aria-label={`Open canister ${canister.name}`}
					on:click={() => navigateToCanisterFiles(canister.id)}
					on:keydown={(e: KeyboardEvent) => e.key === 'Enter' && navigateToCanisterFiles(canister.id)}
					class="focus:outline-none focus:ring-2 focus:ring-blue-500 rounded-xl"
				>
					<!-- Canister Card: Based on 288:110, 288:115 -->
					<Card.Root
						class="border border-white rounded-xl shadow-[0px_4px_4px_0px_rgba(0,0,0,0.25)] cursor-pointer hover:border-blue-500 transition-colors h-full"
					>
						<Card.Content class="p-4 flex flex-col items-center h-full">
							<!-- Placeholder Icon: Based on fill_U5YNZA -->
						<div class="w-16 h-16 bg-gray-500 rounded-full mb-3 flex items-center justify-center">
							<!-- Replace with actual icon/image if available -->
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="32"
								height="32"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
								<polyline points="3.27 6.96 12 12.01 20.73 6.96" />
								<line x1="12" y1="22.08" x2="12" y2="12" />
							</svg>
						</div>
						<!-- Canister Name: Style: style_LZI9YD - Inder, 16px, White, Centered -->
						<p class="font-inder text-base text-center">{canister.name}</p>
							<!-- Optional: Three dots icon (non-functional for now) -->
							<!-- <div class="absolute top-2 right-2 text-gray-400">...</div> -->
						</Card.Content>
					</Card.Root>
				</div>
			{/each}
		</div>
	{:else}
		<!-- Empty State: Based on Figma 280:6 -->
		<div class="flex flex-col items-center justify-center text-center py-16">
			<!-- Logo/Icon: Based on 280:30 -->
			<img src="/logo.svg" alt="Logo" class="w-24 h-24 mb-6 opacity-50" />
			<!-- Text: Style: style_8GQ93Y - Inder, 20px, White, Centered -->
			<p class="font-inder text-xl mb-6">Create a canister to get started.</p>
			<!-- Button is already present in the header -->
		</div>
	{/if}
</div>

<style>
	/* Ensure Inder font is loaded if not globally available */
	/* @import url('https://fonts.googleapis.com/css2?family=Inder&display=swap'); */
	.font-inder {
		font-family: 'Inder', sans-serif;
	}
</style>
