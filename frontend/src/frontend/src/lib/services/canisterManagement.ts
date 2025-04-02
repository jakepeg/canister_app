import { Actor, HttpAgent, type Identity, type ActorSubclass } from '@dfinity/agent'; // Import ActorSubclass
import { AuthClient } from '@dfinity/auth-client'; // Import AuthClient
import { Principal } from '@dfinity/principal';
import {
	ICManagementCanister,
	type CanisterSettings,
	type InstallCodeParams
} from '@dfinity/ic-management';
import { IDL } from '@dfinity/candid';
import { get } from 'svelte/store'; // Import get to read store value
import { authStore, type AuthStateAuthenticated } from '$lib/services/auth'; // To get identity and main actor, import state type
// Corrected import path assuming declarations are at the root src level and _SERVICE is exported
import type { _SERVICE as BackendService } from '../../../../declarations/backend/backend.did';
import { createAgent } from "@dfinity/utils";

// --- Configuration (using SvelteKit env variables) ---
const host = import.meta.env.VITE_HOST;
// Ensure cycles are read as BigInt
const cyclesForNewCanister = BigInt(
	import.meta.env.VITE_CYCLES_FOR_NEW_CANISTER || '2000000000000' // Default if not set
);
const backendWasmPath = '/backend.wasm'; // Assuming Wasm is served here

// --- Result Type ---
export type CreateCanisterResult =
	| { ok: Principal } // Success, returns new canister Principal
	| { err: string }; // Failure, returns error message

// --- Main Creation and Registration Logic ---
export async function createAndRegisterCanister(name: string): Promise<CreateCanisterResult> {
	const authState = get(authStore); // Get the current store value
	if (authState.state !== 'authenticated') {
		return { err: 'User not authenticated' };
	}
	if (!host) {
		return { err: 'VITE_HOST environment variable not set' };
	}

	// Cast the state to the authenticated type to access its properties
	const authenticatedState = authState as AuthStateAuthenticated;
	const authClient = authenticatedState.authClient; // Get AuthClient from state
	if (!authClient) {
		return { err: 'AuthClient not found in store' };
	}
	const identity = authClient.getIdentity(); // Get identity from AuthClient
	if (!identity) {
		return { err: 'Identity not found' }; // Should not happen if authenticated
	}
	const principal = identity.getPrincipal();
	const mainBackendActor = authenticatedState.actor; // Use actor from state
	if (!mainBackendActor) {
		return { err: 'Main backend actor not initialized' };
	}

	console.log(`Using identity: ${principal.toText()}`);
	console.log(`Connecting to IC host: ${host}`);

	// Agent for management canister calls - use the user's identity
	const agent = await createAgent({ identity, host });

	// Fetch root key for local development network ONLY
	if (host.includes('localhost') || host.includes('127.0.0.1')) {
		try {
			console.log('Fetching root key for local replica...');
			await agent.fetchRootKey();
			console.log('Root key fetched.');
		} catch (err: any) {
			console.warn(
				'Could not fetch root key. Network might not be running or requires authentication.',
				err
			);
			// Don't necessarily fail here for local dev
		}
	}

	console.log('Fetching template Wasm module...');
	let templateWasmModule: ArrayBuffer;
	try {
		const response = await fetch(backendWasmPath);
		if (!response.ok) {
			throw new Error(`Failed to fetch Wasm: ${response.statusText}`);
		}
		templateWasmModule = await response.arrayBuffer();
		console.log(`Fetched ${templateWasmModule.byteLength} bytes from ${backendWasmPath}`);
	} catch (err: any) {
		console.error('Error fetching Wasm file:', err);
		return { err: `Error fetching Wasm: ${err.message}` };
	}

	// Create an actor for the Management Canister
	const managementCanister = ICManagementCanister.create({ agent });
	console.log('Management canister actor created.');

	// --- 1. Create Canister ---
	// Define settings according to the expected Candid types
	// controllers is Option<Vec<Principal>>, so Some([principal]) becomes [[principal]]
	// computeAllocation, memoryAllocation, freezingThreshold are Option<Nat>, so None becomes []
	const canisterSettings: CanisterSettings = {
		controllers: [principal.toString()], // Provide the principal in an array for Some(principal)
		computeAllocation: 1n, // Use [] for None (use default)
		memoryAllocation: 1n, // Use [] for None (use default)
		freezingThreshold: 1n, // Use [] for None (use default)
	};

	// createCanister expects settings: Option<CanisterSettings> and amount: Option<Nat>
	const createCanisterArgs = {
		settings: canisterSettings, // Correct: Wrap settings in array for Option<CanisterSettings>
		amount: [cyclesForNewCanister] // Correct: Wrap amount in array for Option<Nat>
	};

	let newCanisterIdRecord;
	try {
		console.log(`Attempting to create canister with ${cyclesForNewCanister} cycles...`);
		// Corrected method name typo
		newCanisterIdRecord = await managementCanister.createCanister(createCanisterArgs);
		console.log(`Canister created successfully: ${newCanisterIdRecord.canister_id.toText()}`);
	} catch (err: any) {
		console.error('Error creating canister:', err);
		return { err: `Error creating canister: ${err.message}` };
	}

	const newCanisterId = newCanisterIdRecord.canister_id;

	// --- 2. Install Code ---
	console.log('Encoding initialization arguments...');
	// Assuming the backend canister's init method takes the creator Principal
	const initArgTypes = [IDL.Principal];
	const initArgValues = [principal];
	let encodedInitArgs: ArrayBuffer;
	try {
		encodedInitArgs = IDL.encode(initArgTypes, initArgValues);
		console.log('Initialization arguments encoded.');
	} catch (err: any) {
		console.error('Error encoding init args:', err);
		return { err: `Error encoding init args: ${err.message}` };
	}

	const installArgs: InstallCodeParams = {
		mode: { install: null },
		canisterId: newCanisterId,
		wasmModule: new Uint8Array(templateWasmModule), // Corrected parameter name typo
		arg: new Uint8Array(encodedInitArgs) // Pass ArrayBuffer directly
	};

	try {
		console.log(`Attempting to install code on canister ${newCanisterId.toText()}...`);
		// Corrected method name typo
		await managementCanister.installCode(installArgs);
		console.log('Code installed successfully!');
	} catch (err: any) {
		console.error('Error installing code:', err);
		// Optional: Consider attempting to stop/delete the partially created canister here
		return { err: `Error installing code: ${err.message}` };
	}

	// --- 3. Register Canister with Main Backend ---
	try {
		console.log(`Registering canister ${newCanisterId.toText()} with name "${name}"...`);
		const registerResult = await mainBackendActor.register_canister(newCanisterId, name);
		if ('Ok' in registerResult) {
			console.log('Canister registered successfully with main backend!');
			return { ok: newCanisterId };
		} else {
			// Handle specific registration errors
			const errorKey = Object.keys(registerResult)[0];
			const errorValue = registerResult[errorKey as keyof typeof registerResult];
			const errorMessage = `Registration failed: ${errorKey}${errorValue ? ` (${errorValue})` : ''}`;
			console.error(errorMessage);
			// Consider if cleanup (delete canister) is needed here
			return { err: errorMessage };
		}
	} catch (err: any) {
		console.error('Error registering canister with main backend:', err);
		// Consider if cleanup (delete canister) is needed here
		return { err: `Error registering canister: ${err.message}` };
	}
}
