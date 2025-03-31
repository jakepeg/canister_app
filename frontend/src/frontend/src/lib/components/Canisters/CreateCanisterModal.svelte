<!-- frontend/src/frontend/src/lib/components/Canisters/CreateCanisterModal.svelte -->
<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Dialog from '$lib/components/ui/dialog'; // Assuming shadcn-svelte dialog components
	import {Label} from "$lib/components/ui/label";
	import { X } from 'lucide-svelte';

	export let open = false; // Control modal visibility from parent

	let canisterName = '';
	let isLoading = false;
	let error = '';

	const dispatch = createEventDispatcher();

	function closeModal() {
		if (isLoading) return;
		open = false;
		canisterName = ''; // Reset form
		error = '';
		dispatch('close'); // Notify parent
	}

	async function handleCreateCanister() {
		if (!canisterName.trim()) {
			error = 'Canister name cannot be empty.';
			return;
		}
		isLoading = true;
		error = '';

		// --- Mock Backend Interaction ---
		console.log('Attempting to create canister:', canisterName);
		await new Promise((resolve) => setTimeout(resolve, 1500)); // Simulate network delay
		const success = Math.random() > 0.2; // Simulate success/failure
		// --- End Mock Backend Interaction ---

		if (success) {
			console.log('Canister created successfully!');
			// TODO: Replace with actual backend call
			// try {
			//   const result = await backendActor.createCanister(canisterName);
			//   if (result.ok) {
			//     dispatch('canisterCreated'); // Notify parent to refresh list
			//     closeModal();
			//   } else {
			//     throw new Error(result.err);
			//   }
			// } catch (err) {
			//   console.error("Failed to create canister:", err);
			//   error = `Failed to create canister: ${err.message}`;
			// } finally {
			//   isLoading = false;
			// }
			dispatch('canisterCreated'); // Notify parent to refresh list (mock success)
			closeModal(); // Close on mock success
		} else {
			console.error('Failed to create canister (mock failure)');
			error = 'Failed to create canister. Please try again. (Mock Error)'; // Mock error
		}
		isLoading = false; // Ensure loading state is reset on mock failure
	}

	// Handle Escape key press to close modal
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeModal();
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

<Dialog.Root bind:open onOpenChange={(val) => !val && closeModal()}>
	<Dialog.Portal>
		<!-- Backdrop: Based on Rectangle 107 fill_RMREMI -->
		<Dialog.Overlay class="fixed inset-0 z-50 bg-black/70" />
		<!-- Modal Container: Based on 289:141 -->
		<Dialog.Content
			class="fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border-2 border-[#0B8CE9] bg-[#1F1F1F] p-6 shadow-lg duration-200 rounded-[21px] text-white font-inder"
			aria-describedby="create-canister-description"
		>
			<!-- Header -->
			<Dialog.Header class="flex justify-between items-center">
				<!-- Title: Style: style_GUBF0I - Inder, 17px, White -->
				<Dialog.Title class="text-lg font-inder">Create New Canister</Dialog.Title>
				<!-- Close Button: Style: style_GUBF0I - Inder, 17px, White -->
				<!-- Reverted to simple button with typed on:click -->
				<button
					on:click={(e: MouseEvent) => closeModal()}
					class="p-1 rounded-full hover:bg-white/10 transition-colors"
					aria-label="Close"
				>
					<X class="h-5 w-5" />
				</button>
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
					<label for="canisterName" class="block text-sm font-inder mb-1">Canister Name</label>
					<!-- Input: Based on Rectangle 93 (289:150) -->
					<Input
						id="canisterName"
						bind:value={canisterName}
						placeholder="Enter canister name"
						class="bg-transparent border border-[#0B8CE9] rounded-[9px] placeholder:text-white/50 placeholder:font-inder placeholder:text-base focus:ring-1 focus:ring-[#0B8CE9]"
						disabled={isLoading}
					/>
				</div>

				<!-- Size Display -->
				<div>
					<!-- Label: Style: style_4O2OYN - Inder, 15px, White -->
					<Label class="block text-sm font-inder mb-1">Size (GB)</Label>
					<!-- Display Field: Based on Rectangle 94 (289:151) -->
					<div
						class="bg-transparent border border-[#0B8CE9] rounded-[9px] px-3 py-2 text-white/50 font-inder text-base"
					>
						500gb <!-- Static value as per spec -->
					</div>
				</div>

				<!-- Setup Cost Display -->
				<div>
					<!-- Label: Style: style_GUBF0I - Inder, 17px, White -->
					<Label class="block text-base font-inder mb-1">Setup Cost:</Label>
					<!-- Value: Needs clarification - Using placeholder -->
					<div class="text-white/80 font-inder text-base">
						TBD <!-- Placeholder - Needs clarification -->
					</div>
				</div>
			</div>

			<!-- Error Message -->
			{#if error}
				<p class="text-red-500 text-sm mt-2">{error}</p>
			{/if}

			<!-- Action Button -->
			<Dialog.Footer class="mt-6">
				<!-- Button: Based on Rectangle 95 (289:154) -->
				<Button
					class="w-full font-inder text-base border border-white rounded-[22px] hover:bg-white/10"
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
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>

<style>
	/* Ensure Inder font is loaded if not globally available */
	/* @import url('https://fonts.googleapis.com/css2?family=Inder&display=swap'); */
	.font-inder {
		font-family: 'Inder', sans-serif;
	}
</style>
