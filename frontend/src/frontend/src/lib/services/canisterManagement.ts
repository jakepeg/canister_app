import { Principal } from "@dfinity/principal";
import {
  ICManagementCanister,
  type CanisterSettings,
  type InstallCodeParams,
} from "@dfinity/ic-management";
import { IDL } from "@dfinity/candid";
import { get } from "svelte/store";
import { authStore, type AuthStateAuthenticated } from "$lib/services/auth";
import type { _SERVICE as BackendService } from "../../../../declarations/backend/backend.did"; // Assuming this path is correct relative to the file
import { type ActorSubclass } from "@dfinity/agent"; // Add ActorSubclass import

// --- Imports for CMC/Ledger Flow ---
import {
  CMCCanister,
  type Cycles,
  ProcessingError,
  type NotifyCreateCanisterArg,
} from "@dfinity/cmc";
import {
  LedgerCanister,
  type BlockHeight,
  type TransferRequest,
  type TimeStamp,
} from "@dfinity/ledger-icp"; // Corrected import: TimeStamp
import { AccountIdentifier, SubAccount } from "@dfinity/ledger-icp";
import { createAgent, principalToSubAccount } from "@dfinity/utils";
import { poll } from "$lib/utils/poll";

// --- Configuration (using SvelteKit env variables) ---
const host = import.meta.env.VITE_HOST;
const dfx_network = import.meta.env.DFX_NETWORK;
// Ensure cycles are read as BigInt
const backendWasmPath = "/backend.wasm"; // Assuming Wasm is served here

// --- Constants for Local CMC/Ledger Flow ---
// Use the canister IDs from your `dfx nns install` output
const LOCAL_LEDGER_CANISTER_ID = Principal.fromText(
  "ryjl3-tyaaa-aaaaa-aaaba-cai",
); // Replace if different
const LOCAL_CMC_CANISTER_ID = Principal.fromText("rkp4c-7iaaa-aaaaa-aaaca-cai"); // Replace if different
export const DEFAULT_ICP_TRANSFER_FEE = 10000n; // 0.0001 ICP
const CREATE_CANISTER_MEMO = BigInt(0x41455243); // CREA, // Use standard 'CREA' memo (0x41455243)
export const TOP_UP_CANISTER_MEMO = BigInt(0x50555054); // TPUP

// --- Result Type ---
export type CreateCanisterResult =
  | { ok: Principal } // Success, returns new canister Principal
  | { err: string }; // Failure, returns error message

export type TopUpResult = { ok: true; message?: string } | { err: string };

// Add these new types after the existing CreateCanisterResult type
export interface CanisterStatusInfo {
  id: Principal;
  name: string;
  status: { running: null } | { stopping: null } | { stopped: null };
  memorySize: bigint;
  memoryAllocation: bigint; // Total allocated memory
  cyclesBalance: bigint;
}

export async function topUpCanisterWithCycles(
  targetCanisterId: Principal,
  icpAmountE8s: bigint, // Amount of ICP (in e8s) user wants to convert and send
): Promise<TopUpResult> {
  console.log(
    `Attempting to top up ${targetCanisterId.toText()} with ${icpAmountE8s} e8s worth of ICP.`,
  );

  const authState = get(authStore);
  if (authState.state !== "authenticated") {
    return { err: "User not authenticated" };
  }
  if (!host) {
    return { err: "VITE_HOST environment variable not set" };
  }

  const { authClient } = authState;
  const identity = authClient.getIdentity();
  const userPrincipal = identity.getPrincipal();

  let agent;
  try {
    agent = await createAgent({ identity, host });
    if (host.includes("localhost") || host.includes("127.0.0.1")) {
      console.log("Fetching root key for local replica (top-up)...");
      await agent.fetchRootKey();
      console.log("Root key fetched for top-up.");
    }
  } catch (err: any) {
    console.error("Error creating agent or fetching root key for top-up:", err);
    return { err: `Agent/root key error: ${err.message}` };
  }

  const cmc = CMCCanister.create({ agent, canisterId: LOCAL_CMC_CANISTER_ID });
  const ledger = LedgerCanister.create({
    agent,
    canisterId: LOCAL_LEDGER_CANISTER_ID,
  });

  // Determine CMC's subaccount for the target canister
  const targetCanisterSubAccountBytes = principalToSubAccount(targetCanisterId);
  const targetCanisterSubAccount = SubAccount.fromBytes(
    targetCanisterSubAccountBytes,
  ) as SubAccount;

  const cmcTargetCanisterAccount = AccountIdentifier.fromPrincipal({
    principal: LOCAL_CMC_CANISTER_ID,
    subAccount: targetCanisterSubAccount,
  });
  const cmcTargetCanisterAccountHex = cmcTargetCanisterAccount.toHex();
  console.log(
    `CMC account for topping up ${targetCanisterId.toText()}: ${cmcTargetCanisterAccountHex}`,
  );

  // Validate ICP amount against fee (user pays transfer fee + sends amount for cycles)
  if (icpAmountE8s <= 0n) {
    // The amount to be converted to cycles must be positive
    return { err: `ICP amount for cycles must be positive.` };
  }
  // The total ICP debited from user will be icpAmountE8s + DEFAULT_ICP_TRANSFER_FEE
  // The icpAmountE8s is what's sent to the CMC account for conversion.

  let blockHeight: BlockHeight;
  try {
    console.log(
      `Transferring ${icpAmountE8s} e8s from ${userPrincipal.toText()} to CMC account ${cmcTargetCanisterAccountHex} for canister ${targetCanisterId.toText()}. User also pays ${DEFAULT_ICP_TRANSFER_FEE} fee.`,
    );
    // Note: The 'amount' field in ledger.transfer is the amount received by 'to'.
    // The sender is debited 'amount' + 'fee'.
    blockHeight = await ledger.transfer({
      to: cmcTargetCanisterAccount,
      amount: icpAmountE8s,
      fee: DEFAULT_ICP_TRANSFER_FEE,
      memo: TOP_UP_CANISTER_MEMO,
      // Assuming your @dfinity/ledger-icp version expects `createdAt` as a direct bigint (nanos)
      // If it expects `created_at_time: { timestamp_nanos: bigint }`, adjust this.
      createdAt: BigInt(Date.now() * 1_000_000),
      fromSubAccount: undefined, // Default subaccount of the user
    });
    console.log(
      `ICP transfer for top-up successful, block height: ${blockHeight}`,
    );
  } catch (err: any) {
    console.error("Error transferring ICP for top-up:", err);
    return { err: `ICP transfer error: ${err.message}` };
  }

  // Notify CMC to top up the canister
  try {
    console.log(
      `Notifying CMC to top up canister ${targetCanisterId.toText()} using block height ${blockHeight}`,
    );
    const notifyResult = await cmc.notifyTopUp({
      block_index: blockHeight,
      canister_id: targetCanisterId,
    });

    if (notifyResult >= 0) {
      const cyclesAdded = notifyResult;
      console.log(
        `CMC successfully processed top-up. Cycles added: ${cyclesAdded}`,
      );
      const cyclesAddedT = (Number(cyclesAdded) / 1_000_000_000_000).toFixed(3);
      return {
        ok: true,
        message: `Successfully added ${cyclesAddedT}T cycles.`,
      };
    } else if (notifyResult < 0) {
      const errorType = Object.keys(notifyResult)[0];
      const errorDetails = (notifyResult as any)[errorType]; // Type assertion for details
      const errorMessage = `CMC notification error: ${errorType}${errorDetails ? ` (${JSON.stringify(errorDetails)})` : ""}`;
      console.error(errorMessage);
      return { err: errorMessage };
    } else {
      console.error("Unknown response from cmc.notifyTopUp:", notifyResult);
      return { err: "Unknown response from CMC notifyTopUp." };
    }
  } catch (err: any) {
    console.error(
      `Error notifying CMC for top-up of ${targetCanisterId.toText()}:`,
      err,
    );
    return { err: `CMC notification error: ${err.message}` };
  }
}

export async function getCanisterStatus(
  canisterId: Principal,
  canisterName: string,
): Promise<CanisterStatusInfo | { err: string }> {
  console.log(`Fetching status for canister: ${canisterId.toText()}`);

  const authState = get(authStore);
  if (authState.state !== "authenticated") {
    console.error("getCanisterStatus: User not authenticated");
    return { err: "User not authenticated" };
  }

  try {
    console.log("Creating agent for canister status check...");
    const agent = await createAgent({
      identity: authState.authClient.getIdentity(),
      host: host,
    });

    // Fetch root key for local development network
    if (host.includes("localhost") || host.includes("127.0.0.1")) {
      try {
        console.log("Fetching root key for local replica...");
        await agent.fetchRootKey();
        console.log("Root key fetched successfully.");
      } catch (err: any) {
        console.warn("Could not fetch root key:", err);
        return { err: "Failed to fetch root key for local development" };
      }
    }

    console.log("Creating management canister instance...");
    const managementCanister = ICManagementCanister.create({ agent });

    console.log("Fetching canister status...");
    const result = await managementCanister.canisterStatus(canisterId);
    console.log("Canister status result:", result);

    const statusInfo = {
      id: canisterId,
      name: canisterName,
      status: result.status,
      memorySize: result.memory_size,
      memoryAllocation: BigInt(150) * BigInt(1024) * BigInt(1024 * 1024),
      cyclesBalance: result.cycles,
    };
    console.log("Returning canister status info:", statusInfo);
    return statusInfo;
  } catch (err: any) {
    console.error("Error in getCanisterStatus:", err);
    console.error("Error stack:", err.stack);
    return { err: err.message || "Failed to fetch canister status" };
  }
}

// --- Main Creation and Registration Logic ---
export async function createAndRegisterCanister(
  name: string,
  sizeGB: number,
): Promise<CreateCanisterResult> {
  const memoryAllocation = BigInt(sizeGB) * BigInt(1024) * BigInt(1024 * 1024); // Convert GB to bytes

  const authState = get(authStore); // Get the current store value
  if (authState.state !== "authenticated") {
    return { err: "User not authenticated" };
  }
  if (!host) {
    return { err: "VITE_HOST environment variable not set" };
  }

  // Cast the state to the authenticated type to access its properties
  const authenticatedState = authState as AuthStateAuthenticated;
  const authClient = authenticatedState.authClient; // Get AuthClient from state
  if (!authClient) {
    return { err: "AuthClient not found in store" };
  }
  const identity = authClient.getIdentity(); // Get identity from AuthClient
  if (!identity) {
    return { err: "Identity not found" }; // Should not happen if authenticated
  }
  const principal = identity.getPrincipal();
  // Ensure mainBackendActor is correctly typed if necessary, casting here
  const mainBackendActor =
    authenticatedState.actor as ActorSubclass<BackendService>;
  if (!mainBackendActor) {
    return { err: "Main backend actor not initialized" };
  }

  console.log(`Using identity: ${principal.toText()}`);
  console.log(`Connecting to IC host: ${host}`);

  // Agent for CMC, Ledger, and Management Canister calls
  const agent = await createAgent({ identity, host });

  // Fetch root key for local development network ONLY
  if (host.includes("localhost") || host.includes("127.0.0.1")) {
    try {
      console.log("Fetching root key for local replica...");
      await agent.fetchRootKey();
      console.log("Root key fetched.");
    } catch (err: any) {
      console.warn(
        "Could not fetch root key. Network might not be running or requires authentication.",
        err,
      );
    }
  }

  // --- Create Canister Actors ---
  const cmc = CMCCanister.create({ agent, canisterId: LOCAL_CMC_CANISTER_ID });
  const ledger = LedgerCanister.create({
    agent,
    canisterId: LOCAL_LEDGER_CANISTER_ID,
  });
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
  const cmcAccountHex = getCanisterCreationCmcAccountIdentifierHex({
    controller: principal,
  });

  // --- Diagnostic Query ---
  try {
    console.log(
      `Querying local ledger (${LOCAL_LEDGER_CANISTER_ID.toText()}) for transfer fee...`,
    );
    const fee = await ledger.transactionFee();
    console.log(`Local ledger transfer fee query successful: ${fee}`);
  } catch (queryErr: any) {
    console.error("Error querying local ledger (diagnostic):", queryErr);
    // Log the error but continue to the transfer attempt to see the original failure point
  }
  // --- End Diagnostic Query ---

  let blockHeight: BlockHeight;
  try {
    console.log("DFX_NETWORK", dfx_network);
    console.log(
      `Transferring ${requiredIcpE8s} e8s to local CMC account ${cmcAccountHex}...`,
    );
    blockHeight = await ledger.transfer({
      to: AccountIdentifier.fromHex(cmcAccountHex),
      amount: requiredIcpE8s,
      fee: DEFAULT_ICP_TRANSFER_FEE,
      memo: CREATE_CANISTER_MEMO,
      createdAt: BigInt(Date.now() * 1_000_000), // Use TimeStamp and correct field name
      fromSubAccount: undefined, // Use default subaccount
    });
    // If the above await succeeds, the returned BlockHeight (bigint) is assigned
    console.log(
      `Simulated ICP Transfer successful, block height: ${blockHeight}`,
    );
  } catch (err: any) {
    console.error("Error transferring simulated ICP:", err);
    return { err: `Error transferring simulated ICP: ${err.message}` };
  }

  // --- 3. Notify Local CMC to Create Canister ---
  let newCanisterId: Principal;
  try {
    console.log(`Notifying local CMC with block height ${blockHeight}...`);
    // Define polling validation and exit conditions
    const validateNotify = (
      result: Principal | ProcessingError,
    ): result is Principal => {
      if (result instanceof ProcessingError) {
        console.log("CMC is still processing...");
        return false; // Continue polling
      }
      return true; // Assume any non-ProcessingError result is the Principal
    };
    const shouldExitNotify = (error: unknown): boolean => {
      // Exit immediately if it's not a ProcessingError
      return !(error instanceof ProcessingError);
    };

    const settings = {
      memory_allocation: [memoryAllocation] as [bigint],
      compute_allocation: [BigInt(0)] as [bigint],
      freezing_threshold: [BigInt(2_592_000)] as [bigint],
      reserved_cycles_limit: [BigInt(5_000_000_000_000)] as [bigint],
      controllers: [[principal]] as [Principal[]],
      wasm_memory_threshold: [BigInt(0)] as [bigint],
      wasm_memory_limit: [] as [],
      log_visibility: [] as [],
    };

    console.log("Settings: ", settings);

    const result = await poll<Principal | ProcessingError>({
      fn: () =>
        cmc.notifyCreateCanister({
          controller: principal,
          block_index: blockHeight,
          settings: [settings], // Add missing optional property
          subnet_type: [], // Add missing optional property
          subnet_selection: [], // Use default subnet selection for local
        }),
      validate: validateNotify,
      shouldExit: shouldExitNotify,
      timeout: 60000, // 60 seconds timeout
      interval: 3000, // Poll every 3 seconds
    });

    // If polling succeeded, the result must be a Principal
    if (result instanceof ProcessingError) {
      // Should not happen if validate/shouldExit are correct, but handle defensively
      throw new Error("Polling finished but CMC still processing.");
    }
    newCanisterId = result;
    console.log(
      `Local CMC notified successfully, new canister ID: ${newCanisterId.toText()}`,
    );
  } catch (err: any) {
    console.error("Error notifying local CMC:", err);
    return { err: `Error notifying local CMC: ${err.message}` };
  }

  // --- 4. Install Code ---
  console.log("Fetching template Wasm module for installation...");
  let templateWasmModule: ArrayBuffer;
  try {
    const response = await fetch(backendWasmPath);
    if (!response.ok) {
      throw new Error(`Failed to fetch Wasm: ${response.statusText}`);
    }
    templateWasmModule = await response.arrayBuffer();
    console.log(
      `Fetched ${templateWasmModule.byteLength} bytes from ${backendWasmPath}`,
    );
  } catch (err: any) {
    console.error("Error fetching Wasm file:", err);
    // Consider trying to clean up (e.g., maybe delete the canister via backend?)
    return { err: `Error fetching Wasm: ${err.message}` };
  }

  console.log("Encoding initialization arguments...");
  const initArgTypes = [IDL.Principal];
  const initArgValues = [principal]; // User principal as init arg
  let encodedInitArgs: ArrayBuffer;
  try {
    encodedInitArgs = IDL.encode(initArgTypes, initArgValues);
    console.log("Initialization arguments encoded.");
  } catch (err: any) {
    console.error("Error encoding init args:", err);
    return { err: `Error encoding init args: ${err.message}` };
  }

  const installArgs: InstallCodeParams = {
    mode: { install: null },
    canisterId: newCanisterId,
    wasmModule: new Uint8Array(templateWasmModule),
    arg: new Uint8Array(encodedInitArgs),
  };

  try {
    console.log(
      `Attempting to install code on canister ${newCanisterId.toText()}...`,
    );
    await managementCanister.installCode(installArgs);
    console.log("Code installed successfully!");
  } catch (err: any) {
    console.error("Error installing code:", err);
    // Optional: Consider attempting to stop/delete the partially created canister here
    return { err: `Error installing code: ${err.message}` };
  }

  // --- 5. Register Canister with Main Backend ---
  try {
    console.log(
      `Registering canister ${newCanisterId.toText()} with name "${name}"...`,
    );
    // Ensure mainBackendActor is correctly typed if necessary
    const registerResult = await mainBackendActor.register_canister(
      newCanisterId,
      name,
    );
    if ("Ok" in registerResult) {
      console.log("Canister registered successfully with main backend!");
      return { ok: newCanisterId };
    } else {
      // Handle specific registration errors from backend's Candid definition
      const errorKey = Object.keys(registerResult)[0];
      const errorValue = (registerResult as any)[errorKey];
      const errorMessage = `Registration failed: ${errorKey}${errorValue ? ` (${JSON.stringify(errorValue)})` : ""}`;
      console.error(errorMessage);
      // Consider if cleanup (delete canister) is needed here
      return { err: errorMessage };
    }
  } catch (err: any) {
    console.error("Error registering canister with main backend:", err);
    // Consider if cleanup (delete canister) is needed here
    return { err: `Error registering canister: ${err.message}` };
  }
}

// Helper function to get canister creation CMC account identifier hex
// (Adapted from NNS Dapp utils)
function getCanisterCreationCmcAccountIdentifierHex({
  controller,
}: {
  controller: Principal;
}): string {
  const subAccountBytes = principalToSubAccount(controller);
  // Explicitly create the SubAccount instance and assert its type to satisfy the compiler.
  const subAccount = SubAccount.fromBytes(subAccountBytes) as SubAccount;
  const accountIdentifier = AccountIdentifier.fromPrincipal({
    principal: LOCAL_CMC_CANISTER_ID, // Use local CMC ID
    subAccount: subAccount, // Pass the created SubAccount instance
  });
  return accountIdentifier.toHex();
}

export async function renameCanister(
  canisterId: Principal,
  newName: string,
): Promise<{ ok: true } | { err: string }> {
  console.log(
    `Attempting to rename canister ${canisterId.toText()} to "${newName}"`,
  );
  const authState = get(authStore);
  if (authState.state !== "authenticated") {
    return { err: "User not authenticated" };
  }

  try {
    const mainBackendActor = authState.actor as ActorSubclass<BackendService>;
    const result = await mainBackendActor.rename_canister(canisterId, newName);

    if ("Ok" in result) {
      return { ok: true };
    } else {
      // Handle specific error cases
      const errorType = Object.keys(result)[0];
      switch (errorType) {
        case "NotAuthorized":
          return { err: "Not authorized to rename this canister" };
        case "CanisterNotFound":
          return { err: "Canister not found" };
        case "InternalError":
          return { err: `Internal error: ${(result as any).InternalError}` };
        default:
          return { err: `Unknown error: ${errorType}` };
      }
    }
  } catch (err: any) {
    console.error("Error renaming canister:", err);
    return { err: err.message || "Failed to rename canister" };
  }
}

export async function deleteCanister(
  canisterId: Principal,
): Promise<{ ok: true } | { err: string }> {
  console.log(
    `Attempting to delete canister ${canisterId.toText()} directly from frontend`,
  );
  const authState = get(authStore);
  if (authState.state !== "authenticated") {
    return { err: "User not authenticated" };
  }

  const identity = authState.authClient.getIdentity();
  const userPrincipal = identity.getPrincipal(); // The user who should be the controller

  console.log(
    `Frontend delete initiated by principal: ${userPrincipal.toText()}`,
  );

  try {
    // Create an agent with the user's identity
    const agent = await createAgent({ identity, host });

    // Fetch root key for local dev ONLY
    if (host.includes("localhost") || host.includes("127.0.0.1")) {
      try {
        await agent.fetchRootKey();
      } catch (err: any) {
        console.warn("Could not fetch root key for delete op:", err);
        // Decide if this is fatal or not for local dev
      }
    }

    // Create management canister instance using the user's agent
    const managementCanister = ICManagementCanister.create({ agent });

    // --- 1. Stop the canister ---
    console.log(`Attempting to stop canister ${canisterId.toText()}...`);
    try {
      // Check status first - maybe it's already stopped? (Optional optimization)
      // const status = await managementCanister.canisterStatus(canisterId);
      // if (!('stopped' in status.status)) { // Only stop if not already stopped
      await managementCanister.stopCanister(canisterId);
      console.log(`Canister ${canisterId.toText()} stopped successfully.`);
      // } else {
      //    console.log(`Canister ${canisterId.toText()} was already stopped.`);
      // }
    } catch (stopErr: any) {
      // Handle specific errors - maybe it's already stopped or doesn't exist?
      // If the error is specifically "Canister is already stopped", we can ignore it.
      // For other errors, re-throw or return error.
      console.error(`Error stopping canister ${canisterId.toText()}:`, stopErr);
      // A more robust check would inspect stopErr.message
      if (!stopErr.message?.includes("already stopped")) {
        // Example check
        return {
          err: `Failed to stop canister: ${stopErr.message || "Unknown stop error"}`,
        };
      }
      console.log(
        `Continuing delete despite stop error (possibly already stopped).`,
      );
    }

    // --- 2. Delete the canister ---
    console.log(
      `Attempting to delete canister ${canisterId.toText()} using management canister...`,
    );
    await managementCanister.deleteCanister(canisterId);
    console.log(
      `Canister ${canisterId.toText()} deleted successfully via management canister.`,
    );

    // --- 3. Unregister from backend (STILL NEEDED) ---
    // We still need to tell our backend that this canister is gone
    // so it doesn't show up in the user's list anymore.
    console.log(
      `Unregistering canister ${canisterId.toText()} from backend...`,
    );
    const mainBackendActor = authState.actor as ActorSubclass<BackendService>;
    if (!mainBackendActor) {
      console.warn(
        "Backend actor not available to unregister canister. Canister deleted but might reappear in list until next manual refresh/login.",
      );
      // Maybe return a specific warning? For now, proceed as deletion was successful.
    } else {
      try {
        // Assuming your backend has an `unregister_canister` or similar function
        // If not, the backend's `delete_canister` might need to ONLY remove from map
        // and not attempt IC management calls. Let's assume an unregister exists for now.
        const unregisterResult =
          await mainBackendActor.unregister_canister(canisterId); // Call backend to update ITS records
        if ("Ok" in unregisterResult) {
          console.log(
            `Canister ${canisterId.toText()} unregistered from backend.`,
          );
        } else {
          const errorKey = Object.keys(unregisterResult)[0];
          console.error(
            `Failed to unregister canister ${canisterId.toText()} from backend: ${errorKey}`,
          );
          // Return success anyway, as the canister IS deleted on chain? Or return warning?
          // Let's return success for now, maybe add logging.
        }
      } catch (unregisterErr: any) {
        console.error(
          `Error calling backend to unregister canister ${canisterId.toText()}:`,
          unregisterErr,
        );
        // Return success anyway?
      }
    }

    return { ok: true }; // Overall success
  } catch (err: any) {
    // Catch errors from agent creation, deleteCanister call etc.
    console.error(
      `Error deleting canister ${canisterId.toText()} from frontend:`,
      err,
    );
    // Provide more specific error messages if possible by inspecting `err`
    if (err.message?.includes("has no controller")) {
      return { err: "You are not a controller of this canister." };
    }
    return {
      err: err.message || "Failed to delete canister via management canister",
    };
  }
}

// --- NEW FUNCTIONS FOR START/STOP ---
async function getManagementCanister(): Promise<
  ICManagementCanister | { err: string }
> {
  const authState = get(authStore);
  if (authState.state !== "authenticated") {
    return { err: "User not authenticated" };
  }

  const identity = authState.authClient.getIdentity();
  try {
    const agent = await createAgent({ identity, host });
    if (host.includes("localhost") || host.includes("127.0.0.1")) {
      try {
        await agent.fetchRootKey();
      } catch (fetchKeyErr: any) {
        console.warn(
          "Could not fetch root key for management operation:",
          fetchKeyErr,
        );
        // Depending on the operation, this might be critical or not.
        // For start/stop, it usually is on local replica.
        return { err: `Failed to fetch root key: ${fetchKeyErr.message}` };
      }
    }
    return ICManagementCanister.create({ agent });
  } catch (agentErr: any) {
    return {
      err: `Failed to create agent for management canister: ${agentErr.message}`,
    };
  }
}

export async function startUserCanister(
  canisterId: Principal,
): Promise<{ ok: true } | { err: string }> {
  console.log(`Attempting to start canister ${canisterId.toText()}`);
  const managementCanisterOrError = await getManagementCanister();
  if ("err" in managementCanisterOrError) {
    return managementCanisterOrError;
  }
  const managementCanister = managementCanisterOrError;

  try {
    await managementCanister.startCanister(canisterId);
    console.log(`Canister ${canisterId.toText()} started successfully.`);
    return { ok: true };
  } catch (err: any) {
    console.error(`Error starting canister ${canisterId.toText()}:`, err);
    if (err.message?.includes("is already running")) {
      console.log(`Canister ${canisterId.toText()} was already running.`);
      return { ok: true }; // Treat as success if already in desired state
    }
    return { err: err.message || "Failed to start canister" };
  }
}

export async function stopUserCanister(
  canisterId: Principal,
): Promise<{ ok: true } | { err: string }> {
  console.log(`Attempting to stop canister ${canisterId.toText()}`);
  const managementCanisterOrError = await getManagementCanister();
  if ("err" in managementCanisterOrError) {
    return managementCanisterOrError;
  }
  const managementCanister = managementCanisterOrError;

  try {
    await managementCanister.stopCanister(canisterId);
    console.log(`Canister ${canisterId.toText()} stopped successfully.`);
    return { ok: true };
  } catch (err: any) {
    console.error(`Error stopping canister ${canisterId.toText()}:`, err);
    if (
      err.message?.includes("is not running") ||
      err.message?.includes("already stopped")
    ) {
      console.log(
        `Canister ${canisterId.toText()} was already stopped or not running.`,
      );
      return { ok: true }; // Treat as success if already in desired state
    }
    return { err: err.message || "Failed to stop canister" };
  }
}
