<!-- frontend/src/frontend/src/routes/canister/[canisterId]/files/+page.svelte -->
<script lang="ts">
	import { page } from '$app/stores';
	import FileList from '$lib/components/Home/FileList.svelte'; // Assuming this is the correct path
	import NotAuthenticated from '$lib/components/Home/NotAuthenticated.svelte';
	import { authStore } from '$lib/services/auth';

	// Extract canisterId from the route parameters
	$: canisterId = $page.params.canisterId;

	// TODO: Potentially pass canisterId to FileList if it needs it
	// TODO: Fetch files specific to this canisterId
</script>

<section class="w-full">
	{#if $authStore.state === 'uninitialized'}
		<div class="flex justify-center items-center h-screen">
			<h1 class="text-xl text-white">Loading...</h1>
		</div>
	{:else if $authStore.state === 'authenticated'}
		<!-- Display the FileList for the specific canister -->
		<!-- Add a title or breadcrumb indicating the current canister -->
		<h1 class="text-lg text-white mb-4 px-4 pt-4">Files in Canister: {canisterId}</h1>
		<FileList auth={$authStore} />
		<!-- Removed canisterId prop as FileList doesn't accept it currently -->
		<!-- TODO: Modify FileList component to use the canisterId from the route ($page.params.canisterId) -->
	{:else}
		<NotAuthenticated />
	{/if}
</section>
