import { Actor, HttpAgent } from "@dfinity/agent";
import { createAgent } from "@dfinity/utils";
import { Principal } from "@dfinity/principal";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import {
  ICManagementCanister,
  CanisterSettings,
  InstallCodeParams,
} from "@dfinity/ic-management";
import { IDL } from "@dfinity/candid";
import fs from "fs";
import path from "path";

// --- Configuration ---
const pemFilePath = process.env.PEM_FILE_PATH;
const host = process.env.IC_HOST;
const templateWasmPath = process.env.TEMPLATE_WASM_PATH;
// Ensure cycles are read as BigInt
const cyclesForNewCanister = BigInt(
  process.env.CYCLES_FOR_NEW_CANISTER || "2000000000000",
); // Default if not set

if (!pemFilePath || !host || !templateWasmPath) {
  console.error(
    "Error: Missing environment variables (PEM_FILE_PATH, IC_HOST, TEMPLATE_WASM_PATH). Check your .env file.",
  );
  process.exit(1);
}

// Resolve paths relative to the script's directory might be safer
const resolvedPemPath = path.resolve(pemFilePath);
const resolvedWasmPath = path.resolve(templateWasmPath);

// --- Helper Function to Load Identity ---
function loadIdentity(filePath) {
  try {
    const pemContent = fs.readFileSync(filePath, "utf8");
    // Basic validation - improve if needed
    if (!pemContent.startsWith("-----BEGIN EC PRIVATE KEY-----")) {
      throw new Error("Invalid PEM file format");
    }
    return Ed25519KeyIdentity.fromPem(pemContent);
  } catch (err) {
    console.error(`Error loading identity from ${filePath}:`, err);
    process.exit(1);
  }
}

// --- Main Duplication Logic ---
async function duplicateBackendCanister() {
  console.log("Loading identity...");
  const identity = loadIdentity(resolvedPemPath);
  const principal = identity.getPrincipal();
  console.log(`Using identity: ${principal.toText()}`);

  console.log(`Connecting to IC host: ${host}`);
  const agent = await createAgent({
    identity: identity,
    host: host,
  });

  if (host === undefined) {
    console.error("Error: Missing IC_HOST environment variable.");
    return;
  }

  // Fetch root key for local development network ONLY
  if (host.includes("localhost") || host.includes("127.0.0.1")) {
    try {
      console.log("Fetching root key for local replica...");
      await agent.fetchRootKey();
      console.log("Root key fetched.");
    } catch (err) {
      console.warn(
        "Could not fetch root key. Network might not be running or requires authentication.",
      );
    }
  }

  console.log("Reading template Wasm module...");
  let templateWasmModule;
  try {
    templateWasmModule = fs.readFileSync(resolvedWasmPath);
    console.log(
      `Read ${templateWasmModule.length} bytes from ${resolvedWasmPath}`,
    );
  } catch (err) {
    console.error(`Error reading Wasm file:`, err);
    process.exit(1);
  }

  // Create an actor for the Management Canister
  const managementCanister = ICManagementCanister.create({ agent });
  console.log("Management canister actor created.");

  // --- 1. Create Canister ---
  // Define the specific settings object with the CanisterSettings type
  // This tells TypeScript to check if the object structure matches the type.
  const specificSettings: CanisterSettings = {
    // Using Option type: [value] for Some, [] for None for nested fields
    controllers: [principal], // Provide the principal in an array for Some(principal)
    computeAllocation: 0n, // Use [] for None (use default)
    memoryAllocation: 0n, // Use [] for None (use default)
    freezingThreshold: 0n, // Use [] for None (use default)
    // reserved_cycles_limit: [], // Optional: Use [] for None
    // wasm_memory_limit: [],     // Optional: Use [] for None
  };

  // Define the arguments for the create_canister call.
  // The 'settings' field itself is an Option<CanisterSettings>,
  // so we wrap our 'specificSettings' object in an array: [specificSettings]
  const createCanisterArgs = {
    settings: specificSettings, // Type: [] | [CanisterSettings]
    amount: [cyclesForNewCanister], // Type: [] | [bigint] (Optional amount)
    // sender_canister_version: []      // Optional field
  };

  let newCanisterIdRecord;
  try {
    console.log(
      `Attempting to create canister with ${cyclesForNewCanister} cycles...`,
    );
    newCanisterIdRecord =
      await managementCanister.createCanister(createCanisterArgs);
    console.log(
      `Canister created successfully: ${newCanisterIdRecord.canister_id.toText()}`,
    );
  } catch (err) {
    console.error("Error creating canister:", err);
    process.exit(1);
  }

  const newCanisterId = newCanisterIdRecord.canister_id;

  // --- 2. Install Code ---

  // Encode the initialization arguments for the template canister's init(creator: Principal) function
  // The argument is the Principal of the identity running this script.
  console.log("Encoding initialization arguments...");
  const initArgTypes = [IDL.Principal]; // The types expected by the 'init' function
  const initArgValues = [principal]; // The values for the 'init' function arguments
  const encodedInitArgs = IDL.encode(initArgTypes, initArgValues);
  console.log("Initialization arguments encoded.");

  // Define the install arguments object and apply the InstallCodeParams type
  const installArgs: InstallCodeParams = {
    mode: { install: null }, // Variant type for install mode
    canister_id: newCanisterId, // Principal type
    // wasm_module is Vec<Nat8>. Libraries often accept Uint8Array/Buffer directly,
    // or require conversion to number[]. The spread `[...]` does the conversion.
    wasm_module: [...templateWasmModule],
    // arg is Vec<Nat8>. Convert ArrayBuffer from IDL.encode to number[]
    // Buffer.from() handles ArrayBuffer.
    arg: [...Buffer.from(encodedInitArgs)],
    // sender_canister_version: [] // Optional field
  };

  try {
    console.log(
      `Attempting to install code on canister ${newCanisterId.toText()}...`,
    );
    await managementCanister.installCode(installArgs);
    console.log("\n✅ Code installed successfully!");
    console.log(`✅ New Backend Canister Principal: ${newCanisterId.toText()}`);
    console.log(`   Controlled by: ${principal.toText()}`);
    console.log(`   Initial cycles provided: ${cyclesForNewCanister}`);
  } catch (err) {
    console.error("\n❌ Error installing code:", err);
    // Optional: Consider attempting to stop/delete the partially created canister here if needed
    // await managementCanister.stop_canister({ canister_id: newCanisterId });
    // await managementCanister.delete_canister({ canister_id: newCanisterId });
    // Note: Deleting requires the canister to be stopped and empty.
    process.exit(1);
  }
}

// Run the duplication process
duplicateBackendCanister().catch((error) => {
  console.error("\n🚨 Unhandled error during duplication:", error);
  process.exit(1);
});
