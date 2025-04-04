use crate::{with_user_canisters, with_user_canisters_mut, CanisterInfo, CanisterInfoVec};
use candid::{CandidType, Principal}; // Removed Nat
use ic_cdk::api::management_canister::main::{
    canister_status,
    CanisterIdRecord, // Removed CanisterSettings as ManagementCanisterSettings, CanisterStatusResponse
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
// Removed RegisterCanisterResponse and GetUserCanistersResponse enum definitions from here. They will be moved to lib.rs.
// Import the necessary types from lib.rs
use crate::{CreateCanisterResponse, GetUserCanistersResponse, RegisterCanisterResponse};

// Include the generated code containing the Wasm bytes
include!(concat!(env!("OUT_DIR"), "/backend_wasm.rs"));

// Placeholder for get_user_canisters
// Removed #[query] macro
pub fn get_user_canisters() -> GetUserCanistersResponse {
    // Added pub
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return GetUserCanistersResponse::NotAuthenticated;
    }

    let canisters = with_user_canisters(|map| {
        map.get(&caller)
            .map(|info_vec| info_vec.0.clone())
            .unwrap_or_default()
    });

    GetUserCanistersResponse::Ok(canisters)
}

// Placeholder for register_canister
// Removed #[update] macro
pub async fn register_canister(canister_id: Principal, name: String) -> RegisterCanisterResponse {
    // Added pub
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        // Should not happen for update calls, but good practice
        return RegisterCanisterResponse::NotAuthorized;
    }

    // --- Verification Step ---
    match check_caller_is_controller(canister_id, caller).await {
        Ok(true) => {
            // Caller is a controller, proceed
            ic_cdk::println!(
                "Caller {} verified as controller for {}",
                caller,
                canister_id
            );
        }
        Ok(false) => {
            ic_cdk::println!("Caller {} is NOT a controller for {}", caller, canister_id);
            return RegisterCanisterResponse::NotAuthorized;
        }
        Err(e) => {
            ic_cdk::println!(
                "Error verifying controller status for {}: {}",
                canister_id,
                e
            );
            return RegisterCanisterResponse::VerificationFailed(e);
        }
    }
    // --- End Verification ---

    // --- Add to map ---
    let new_canister_info = CanisterInfo {
        id: canister_id,
        name,
    };

    let result = with_user_canisters_mut(|map| {
        let mut current_list = map.get(&caller).unwrap_or_default();

        // Check if already registered
        if current_list.0.iter().any(|c| c.id == canister_id) {
            return Err(RegisterCanisterResponse::AlreadyRegistered);
        }

        current_list.0.push(new_canister_info);
        map.insert(caller, current_list);
        Ok(())
    });

    match result {
        Ok(_) => RegisterCanisterResponse::Ok,
        Err(e) => e, // Return AlreadyRegistered error
    }
}

// --- Function to Create a New File Canister ---

// Placeholder cycles amount - adjust as needed (e.g., 1T cycles)
const CREATE_CYCLES: u128 = 1_000_000_000_000;
// Placeholder cycles amount for installation - adjust as needed
const INSTALL_CYCLES: u128 = 1_000_000_000_000;

// Removed #[update] macro for now, will be added in main.rs
pub async fn create_new_file_canister(name: String) -> CreateCanisterResponse {
    // Added pub
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return CreateCanisterResponse::Err("Anonymous caller not allowed".to_string());
    }

    // --- 1. Get Wasm Module ---
    // Use the embedded Wasm bytes from the build script
    let wasm_module: Vec<u8> = BACKEND_WASM.to_vec();
    if BACKEND_WASM.is_empty() {
        // This check is important if the build script created an empty placeholder
        return CreateCanisterResponse::Err(
            "Backend Wasm module is empty or not found during build.".to_string(),
        );
    }

    // --- 2. Create Canister ---
    let create_args = ic_cdk::api::management_canister::main::CreateCanisterArgument {
        settings: Some(ic_cdk::api::management_canister::main::CanisterSettings {
            controllers: Some(vec![caller, ic_cdk::id()]), // Caller and this canister are controllers
            compute_allocation: None,                      // Use default
            memory_allocation: None,                       // Use default
            freezing_threshold: None,                      // Use default
            reserved_cycles_limit: None,                   // Added field
            log_visibility: None,                          // Added field
            wasm_memory_limit: None,                       // Added field
        }),
    };

    let create_result =
        ic_cdk::api::management_canister::main::create_canister(create_args, CREATE_CYCLES).await;

    let new_canister_principal = match create_result {
        Ok((canister_id_record,)) => {
            ic_cdk::println!("Canister created: {}", canister_id_record.canister_id);
            canister_id_record.canister_id
        }
        Err((code, msg)) => {
            let error_msg = format!("Failed to create canister ({:?}): {}", code, msg);
            ic_cdk::println!("{}", error_msg);
            // Attempt to clean up? Maybe not necessary if creation failed before charging.
            return CreateCanisterResponse::Err(error_msg);
        }
    };

    // --- 3. Install Code ---
    let install_args = ic_cdk::api::management_canister::main::InstallCodeArgument {
        mode: ic_cdk::api::management_canister::main::CanisterInstallMode::Install,
        canister_id: new_canister_principal,
        wasm_module: wasm_module.clone(), // Use the retrieved Wasm
        arg: vec![],                      // No initialization arguments for the new canister
    };

    // Use call_with_payment128 for installing code with cycles
    let install_result: Result<(), _> = ic_cdk::api::call::call_with_payment128(
        Principal::management_canister(),
        "install_code",
        (install_args,), // Arguments must be in a tuple
        INSTALL_CYCLES,
    )
    .await;

    match install_result {
        Ok(()) => {
            // Match Ok(()) for CallResult
            ic_cdk::println!("Code installed successfully on {}", new_canister_principal);
            // --- 4. Store Association ---
            let new_canister_info = CanisterInfo {
                id: new_canister_principal,
                name: name.clone(),
            };

            let store_result = with_user_canisters_mut(|map| {
                let mut current_list = map.get(&caller).unwrap_or_default();
                // Avoid duplicates just in case, though unlikely here
                if !current_list
                    .0
                    .iter()
                    .any(|c| c.id == new_canister_principal)
                {
                    current_list.0.push(new_canister_info);
                    map.insert(caller, current_list);
                }
            });
            ic_cdk::println!(
                "Association stored for caller {} and canister {}",
                caller,
                new_canister_principal
            );

            CreateCanisterResponse::Ok(new_canister_principal)
        }
        Err((code, msg)) => {
            let error_msg = format!(
                "Failed to install code on {} ({:?}): {}",
                new_canister_principal, code, msg
            );
            ic_cdk::println!("{}", error_msg);
            // TODO: Consider attempting to delete the canister if installation fails.
            // Requires another management call: delete_canister
            CreateCanisterResponse::Err(error_msg)
        }
    }
}
// --- End Create Function ---

// --- Helper function for verification ---
async fn check_caller_is_controller(
    canister_to_check: Principal,
    expected_controller: Principal,
) -> Result<bool, String> {
    let args = CanisterIdRecord {
        canister_id: canister_to_check,
    };

    match canister_status(args).await {
        Ok((status,)) => {
            // Check if expected_controller is in the controllers list
            Ok(status.settings.controllers.contains(&expected_controller))
        }
        Err((code, msg)) => Err(format!(
            "Failed to get canister status ({:?}): {}",
            code, msg
        )),
    }
}
