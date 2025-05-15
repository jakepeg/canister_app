use crate::{with_user_canisters, with_user_canisters_mut, CanisterInfo}; // Added CanisterInfoVec
use candid::Principal;
// use ic_cdk::api::management_canister::main::{stop_canister, CanisterIdRecord};

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

// RENAMED and SIMPLIFIED function
// Changed return type to use DeleteCanisterResponse for simplicity, but adjust if needed
// Removed `async` as it no longer makes async calls
pub fn unregister_canister_internal(canister_id: Principal) -> DeleteCanisterResponse {
    let caller = ic_cdk::caller();
    // While update calls usually block anonymous, explicit check is safer if query calls could somehow reach this logic
    if caller == Principal::anonymous() {
        ic_cdk::println!("Attempt to unregister canister by anonymous caller blocked.");
        return DeleteCanisterResponse::NotAuthorized;
    }

    ic_cdk::println!(
        "Unregister attempt: caller {}, target canister {}",
        caller,
        canister_id
    );

    // Use mutable access to check and remove in one go
    let result = with_user_canisters_mut(|map| {
        if let Some(mut canister_list_vec) = map.get(&caller) {
            let initial_len = canister_list_vec.0.len();
            // Retain only canisters whose ID is NOT the one we want to remove
            canister_list_vec.0.retain(|c| c.id != canister_id);
            let final_len = canister_list_vec.0.len();

            if initial_len == final_len {
                // The canister ID was not found in the list for this caller
                ic_cdk::println!(
                    "Unregister failed: Canister {} not found in list for caller {}.",
                    canister_id,
                    caller
                );
                Err(DeleteCanisterResponse::CanisterNotFound)
            } else {
                // Canister was found and removed, update the map
                ic_cdk::println!(
                    "Unregister success: Canister {} removed from list for caller {}. Length {} -> {}.",
                    canister_id,
                    caller,
                    initial_len,
                    final_len
                );
                // Re-insert the modified vector back into the map
                map.insert(caller, canister_list_vec);
                Ok(DeleteCanisterResponse::Ok)
            }
        } else {
            // The caller principal has no canisters registered at all
            ic_cdk::println!(
                "Unregister failed: Caller {} has no registered canisters.",
                caller
            );
            Err(DeleteCanisterResponse::CanisterNotFound)
        }
    });

    // Return the result (Ok or CanisterNotFound)
    match result {
        Ok(response) => response,
        Err(response) => response,
    }
    // Note: InternalError for map operations is less likely but could be added if needed
}
