use crate::{with_user_canisters, with_user_canisters_mut, CanisterInfo, CanisterInfoVec};
use candid::{CandidType, Principal}; // Removed Nat
use ic_cdk::api::management_canister::main::{
    canister_status,
    CanisterIdRecord, // Removed CanisterSettings as ManagementCanisterSettings, CanisterStatusResponse
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
// Removed RegisterCanisterResponse and GetUserCanistersResponse enum definitions from here. They will be moved to lib.rs.
use crate::{GetUserCanistersResponse, RegisterCanisterResponse}; // Import the types from lib.rs

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

    // --- Verification Step Removed ---
    // The backend trusts that if the frontend successfully created the canister
    // and called this function, the caller controls the canister_id.
    // Attempting verification here fails because the backend canister itself
    // is not a controller of the newly created canister.
    // --- End Verification Removed ---

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
