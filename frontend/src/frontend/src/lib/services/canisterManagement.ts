import { Principal } from '@dfinity/principal';
import {
	ICManagementCanister,
	type CanisterSettings,
	type InstallCodeParams
} from '@dfinity/ic-management';
import { IDL } from '@dfinity/candid';
import { get } from 'svelte/store';
import { authStore, type AuthStateAuthenticated } from '$lib/services/auth';
import type { _SERVICE as BackendService } from '../../../../declarations/backend/backend.did'; // Assuming this path is correct relative to the file
import { type ActorSubclass } from '@dfinity/agent'; // Add ActorSubclass import

// --- Imports for CMC/Ledger Flow ---
import { CMCCanister, type Cycles, ProcessingError } from '@dfinity/cmc';
import { LedgerCanister, type BlockHeight, type TransferRequest, type TimeStamp } from '@dfinity/ledger-icp'; // Corrected import: TimeStamp
import { AccountIdentifier, SubAccount } from '@dfinity/ledger-icp';
import { createAgent, principalToSubAccount } from '@dfinity/utils';
import { poll } from '$lib/utils/poll';

// --- Configuration (using SvelteKit env variables) ---
const host = import.meta.env.VITE_HOST;
const dfx_network = import.meta.env.DFX_NETWORK;
// Ensure cycles are read as BigInt
const backendWasmPath = '/backend.wasm'; // Assuming Wasm is served here

// --- Constants for Local CMC/Ledger Flow ---
// Use the canister IDs from your `dfx nns install` output
const LOCAL_LEDGER_CANISTER_ID = Principal.fromText('ryjl3-tyaaa-aaaaa-aaaba-cai'); // Replace if different
const LOCAL_CMC_CANISTER_ID = Principal.fromText('rkp4c-7iaaa-aaaaa-aaaca-cai'); // Replace if different
const DEFAULT_ICP_TRANSFER_FEE = 10000n; // 0.0001 ICP
const CREATE_CANISTER_MEMO =  BigInt(0x41455243); // CREA, // Use standard 'CREA' memo (0x41455243)

// --- Result Type ---
export type CreateCanisterResult =
	| { ok: Principal } // Success, returns new canister Principal
	| { err: string }; // Failure, returns error message

// Add these new types after the existing CreateCanisterResult type
export interface CanisterStatusInfo {
    id: Principal;
    name: string;
    status: { running: null } | { stopping: null } | { stopped: null };
    memorySize: bigint;
    memoryAllocation: bigint; // Total allocated memory
    cyclesBalance: bigint;
}

export async function getCanisterStatus(canisterId: Principal, canisterName: string): Promise<CanisterStatusInfo | { err: string }> {
    const authState = get(authStore);
    if (authState.state !== 'authenticated') {
        return { err: 'User not authenticated' };
    }

    try {
        const agent = await createAgent({ 
            identity: authState.authClient.getIdentity(), 
            host: host 
        });

        const managementCanister = ICManagementCanister.create({ agent });
        
        const result = await managementCanister.canisterStatus(canisterId);
        
        const mainBackendActor = authState.actor as ActorSubclass<BackendService>;
        
        return {
            id: canisterId,
            name: canisterName,
            status: result.status,
            memorySize: result.memory_size,
            memoryAllocation: BigInt(150) * BigInt(1024) * BigInt(1024 * 1024),
            cyclesBalance: result.cycles
        };
    } catch (err: any) {
        console.error('Error fetching canister status:', err);
        return { err: err.message || 'Failed to fetch canister status' };
    }
}

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
	// Ensure mainBackendActor is correctly typed if necessary, casting here
	const mainBackendActor = authenticatedState.actor as ActorSubclass<BackendService>;
	if (!mainBackendActor) {
		return { err: 'Main backend actor not initialized' };
	}

	console.log(`Using identity: ${principal.toText()}`);
	console.log(`Connecting to IC host: ${host}`);

	// Agent for CMC, Ledger, and Management Canister calls
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
		}
	}

	// --- Create Canister Actors ---
	const cmc = CMCCanister.create({ agent, canisterId: LOCAL_CMC_CANISTER_ID });
	const ledger = LedgerCanister.create({ agent, canisterId: LOCAL_LEDGER_CANISTER_ID });
	const managementCanister = ICManagementCanister.create({ agent }); // Still needed for installCode

	// --- 1. Calculate ICP Amount (Simulated for Local) ---
	// For local testing, we often don't need the exact cycle conversion.
	// We just need enough simulated ICP to cover the transfer fee.
	// Let's assume the local CMC doesn't strictly enforce cycle amounts based on ICP.
	// We'll send a nominal amount + fee.
	const simulatedIcpAmount = 10000000n; // 0.1 ICP (example)
	const requiredIcpE8s = simulatedIcpAmount + DEFAULT_ICP_TRANSFER_FEE;
	console.log(`Simulated ICP to transfer (e8s): ${requiredIcpE8s}`);

	// --- 2. Transfer Simulated ICP to Local CMC ---
	const cmcAccountHex = getCanisterCreationCmcAccountIdentifierHex({ controller: principal });

	// --- Diagnostic Query ---
	try {
		console.log(`Querying local ledger (${LOCAL_LEDGER_CANISTER_ID.toText()}) for transfer fee...`);
		const fee = await ledger.transactionFee();
		console.log(`Local ledger transfer fee query successful: ${fee}`);
	} catch (queryErr: any) {
		console.error('Error querying local ledger (diagnostic):', queryErr);
		// Log the error but continue to the transfer attempt to see the original failure point
	}
	// --- End Diagnostic Query ---

	let blockHeight: BlockHeight;
	try {
		console.log("DFX_NETWORK", dfx_network);
		console.log(`Transferring ${requiredIcpE8s} e8s to local CMC account ${cmcAccountHex}...`);
		blockHeight = await ledger.transfer({
			to: AccountIdentifier.fromHex(cmcAccountHex),
			amount: requiredIcpE8s,
			fee: DEFAULT_ICP_TRANSFER_FEE,
			memo: CREATE_CANISTER_MEMO,
			createdAt: BigInt(Date.now() * 1_000_000)  , // Use TimeStamp and correct field name
			fromSubAccount: undefined, // Use default subaccount
		});
		// If the above await succeeds, the returned BlockHeight (bigint) is assigned
		console.log(`Simulated ICP Transfer successful, block height: ${blockHeight}`);
	} catch (err: any) {
		console.error('Error transferring simulated ICP:', err);
		return { err: `Error transferring simulated ICP: ${err.message}` };
	}

	// --- 3. Notify Local CMC to Create Canister ---
	let newCanisterId: Principal;
	try {
		console.log(`Notifying local CMC with block height ${blockHeight}...`);
		// Define polling validation and exit conditions
		const validateNotify = (result: Principal | ProcessingError): result is Principal => {
			if (result instanceof ProcessingError) {
				console.log('CMC is still processing...');
				return false; // Continue polling
			}
			return true; // Assume any non-ProcessingError result is the Principal
		};
		const shouldExitNotify = (error: unknown): boolean => {
			// Exit immediately if it's not a ProcessingError
			return !(error instanceof ProcessingError);
		};

		const result = await poll<Principal | ProcessingError>({
			fn: () =>
				cmc.notifyCreateCanister({
					controller: principal,
					block_index: blockHeight,
					settings: [], // Add missing optional property
					subnet_type: [], // Add missing optional property
					subnet_selection: [] // Use default subnet selection for local
				}),
			validate: validateNotify,
			shouldExit: shouldExitNotify,
			timeout: 60000, // 60 seconds timeout
			interval: 3000, // Poll every 3 seconds
		});

		// If polling succeeded, the result must be a Principal
		if (result instanceof ProcessingError) {
			// Should not happen if validate/shouldExit are correct, but handle defensively
			throw new Error('Polling finished but CMC still processing.');
		}
		newCanisterId = result;
		console.log(`Local CMC notified successfully, new canister ID: ${newCanisterId.toText()}`);

	} catch (err: any) {
		console.error('Error notifying local CMC:', err);
		return { err: `Error notifying local CMC: ${err.message}` };
	}

	// --- 4. Install Code ---
	console.log('Fetching template Wasm module for installation...');
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
		// Consider trying to clean up (e.g., maybe delete the canister via backend?)
		return { err: `Error fetching Wasm: ${err.message}` };
	}

	console.log('Encoding initialization arguments...');
	const initArgTypes = [IDL.Principal];
	const initArgValues = [principal]; // User principal as init arg
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
		wasmModule: new Uint8Array(templateWasmModule),
		arg: new Uint8Array(encodedInitArgs)
	};

	try {
		console.log(`Attempting to install code on canister ${newCanisterId.toText()}...`);
		await managementCanister.installCode(installArgs);
		console.log('Code installed successfully!');
	} catch (err: any) {
		console.error('Error installing code:', err);
		// Optional: Consider attempting to stop/delete the partially created canister here
		return { err: `Error installing code: ${err.message}` };
	}

	// --- 5. Register Canister with Main Backend ---
	try {
		console.log(`Registering canister ${newCanisterId.toText()} with name "${name}"...`);
		// Ensure mainBackendActor is correctly typed if necessary
		const registerResult = await mainBackendActor.register_canister(newCanisterId, name);
		if ('Ok' in registerResult) {
			console.log('Canister registered successfully with main backend!');
			return { ok: newCanisterId };
		} else {
			// Handle specific registration errors from backend's Candid definition
			const errorKey = Object.keys(registerResult)[0];
			const errorValue = (registerResult as any)[errorKey];
			const errorMessage = `Registration failed: ${errorKey}${errorValue ? ` (${JSON.stringify(errorValue)})` : ''}`;
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

// Helper function to get canister creation CMC account identifier hex
// (Adapted from NNS Dapp utils)
function getCanisterCreationCmcAccountIdentifierHex({ controller }: { controller: Principal }): string {
	const subAccountBytes = principalToSubAccount(controller);
	// Explicitly create the SubAccount instance and assert its type to satisfy the compiler.
	const subAccount = SubAccount.fromBytes(subAccountBytes) as SubAccount;
	const accountIdentifier = AccountIdentifier.fromPrincipal({
		principal: LOCAL_CMC_CANISTER_ID, // Use local CMC ID
		subAccount: subAccount // Pass the created SubAccount instance
	});
	return accountIdentifier.toHex();
}
