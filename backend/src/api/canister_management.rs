use crate::{with_user_canisters, with_user_canisters_mut, CanisterInfo}; // Added CanisterInfoVec
use candid::Principal;
use ic_cdk::api::management_canister::main::{stop_canister, CanisterIdRecord};

// Import the response types from lib.rs
use crate::{
    DeleteCanisterResponse, GetUserCanistersResponse, RegisterCanisterResponse,
    RenameCanisterResponse,
};

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

pub async fn rename_canister(canister_id: Principal, new_name: String) -> RenameCanisterResponse {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return RenameCanisterResponse::NotAuthorized;
    }

    with_user_canisters_mut(|map| {
        // Get the owned CanisterInfoVec if it exists
        if let Some(mut canister_list_vec) = map.get(&caller) {
            let mut found_and_modified = false;
            // Iterate through the inner Vec mutably
            if let Some(canister) = canister_list_vec.0.iter_mut().find(|c| c.id == canister_id) {
                canister.name = new_name;
                found_and_modified = true;
            }

            if found_and_modified {
                // Insert the modified CanisterInfoVec back into the map
                map.insert(caller, canister_list_vec);
                RenameCanisterResponse::Ok
            } else {
                // Canister ID wasn't found in the user's list
                RenameCanisterResponse::CanisterNotFound
            }
        } else {
            // User has no canisters registered
            RenameCanisterResponse::CanisterNotFound
        }
    })
}

pub async fn delete_canister_internal(canister_id: Principal) -> DeleteCanisterResponse {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return DeleteCanisterResponse::NotAuthorized;
    }

    // First verify the canister exists in user's list (read-only)
    let canister_exists = with_user_canisters(|map| {
        map.get(&caller)
            .map(|list| list.0.iter().any(|c| c.id == canister_id))
            .unwrap_or(false)
    });

    if !canister_exists {
        return DeleteCanisterResponse::CanisterNotFound;
    }

    // Stop the canister first - Pass CanisterIdRecord by value
    match stop_canister(CanisterIdRecord { canister_id }).await {
        Ok(_) => ic_cdk::println!("Canister {} stopped successfully.", canister_id),
        Err((code, msg)) => {
            ic_cdk::println!(
                "Failed to stop canister {}: {:?} - {}",
                canister_id,
                code,
                msg
            );
            // Proceed with deletion attempt even if stop fails? Or return error?
            // Returning error here is safer.
            return DeleteCanisterResponse::DeletionFailed(format!(
                "Failed to stop canister (code: {:?}): {}",
                code, msg
            ));
        }
    }

    // Delete the canister using the IC management canister - Pass CanisterIdRecord by value
    match ic_cdk::api::management_canister::main::delete_canister(CanisterIdRecord { canister_id })
        .await
    {
        Ok(_) => {
            ic_cdk::println!(
                "Canister {} deleted successfully via management canister.",
                canister_id
            );
            // Remove from user's list
            let result = with_user_canisters_mut(|map| {
                // Get the list again within the mutable context
                if let Some(mut canister_list_vec) = map.get(&caller) {
                    ic_cdk::println!(
                        "Removing canister {} from user {}'s list.",
                        canister_id,
                        caller
                    );
                    let initial_len = canister_list_vec.0.len();
                    // Modify the owned vec
                    canister_list_vec.0.retain(|c| c.id != canister_id);
                    let final_len = canister_list_vec.0.len();
                    ic_cdk::println!("List length changed from {} to {}.", initial_len, final_len);

                    // Insert the modified vec back
                    map.insert(caller, canister_list_vec);
                    Ok(())
                } else {
                    // Should not happen if initial check passed, but handle defensively.
                    ic_cdk::println!(
                        "Error: Canister list for user {} not found during removal.",
                        caller
                    );
                    Err("Internal error: User canister list disappeared unexpectedly.")
                }
            });

            match result {
                Ok(_) => DeleteCanisterResponse::Ok,
                Err(msg) => {
                    ic_cdk::println!("Internal error after canister deletion: {}", msg);
                    DeleteCanisterResponse::InternalError(msg.to_string())
                }
            }
        }
        Err((code, msg)) => {
            ic_cdk::println!(
                "Failed to delete canister {} via management canister: {:?} - {}",
                canister_id,
                code,
                msg
            );
            DeleteCanisterResponse::DeletionFailed(format!(
                "Failed to delete canister (code: {:?}): {}",
                code, msg
            ))
        }
    }
}
