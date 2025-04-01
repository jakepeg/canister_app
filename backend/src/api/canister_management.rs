use crate::{CanisterInfo, CanisterInfoVec, with_user_canisters, with_user_canisters_mut};
use candid::{CandidType, Principal, Nat};
use ic_cdk::api::management_canister::main::{CanisterIdRecord, CanisterStatusResponse, CanisterSettings as ManagementCanisterSettings, canister_status};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};

// Define response types for the new methods
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RegisterCanisterResponse {
    Ok,
    NotAuthorized, // If caller doesn't control the canister_id
    VerificationFailed(String), // If canister_status call fails
    AlreadyRegistered,
    InternalError(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetUserCanistersResponse {
    Ok(Vec<CanisterInfo>),
    NotAuthenticated,
}


// Placeholder for get_user_canisters
#[query]
fn get_user_canisters() -> GetUserCanistersResponse {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return GetUserCanistersResponse::NotAuthenticated;
    }

    let canisters = with_user_canisters(|map| {
        map.get(&caller).map(|info_vec| info_vec.0.clone()).unwrap_or_default()
    });

    GetUserCanistersResponse::Ok(canisters)
}

// Placeholder for register_canister
#[update]
async fn register_canister(canister_id: Principal, name: String) -> RegisterCanisterResponse {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        // Should not happen for update calls, but good practice
        return RegisterCanisterResponse::NotAuthorized;
    }

        // --- Verification Step ---
    match check_caller_is_controller(canister_id, caller).await {
        Ok(true) => {
            // Caller is a controller, proceed
            ic_cdk::println!("Caller {} verified as controller for {}", caller, canister_id);
        }
        Ok(false) => {
            ic_cdk::println!("Caller {} is NOT a controller for {}", caller, canister_id);
            return RegisterCanisterResponse::NotAuthorized;
        }
        Err(e) => {
             ic_cdk::println!("Error verifying controller status for {}: {}", canister_id, e);
            return RegisterCanisterResponse::VerificationFailed(e);
        }
    }
    // --- End Verification ---

    // --- Add to map ---
    let new_canister_info = CanisterInfo { id: canister_id, name };

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
async fn check_caller_is_controller(canister_to_check: Principal, expected_controller: Principal) -> Result<bool, String> {
    let args = CanisterIdRecord { canister_id: canister_to_check };

    match canister_status(args).await {
        Ok((status,)) => {
            // Check if expected_controller is in the controllers list
            Ok(status.settings.controllers.contains(&expected_controller))
        }
        Err((code, msg)) => {
            Err(format!("Failed to get canister status ({:?}): {}", code, msg))
        }
    }
}
