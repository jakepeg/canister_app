// ic-docutrack/backend/src/main.rs
use backend::*; // Imports all necessary types from lib.rs
use candid::Principal;
use ic_cdk::api::caller;
use ic_cdk_macros::{post_upgrade, pre_upgrade, query, update};

// --- User Management ---
#[update]
fn set_user(username: String, public_key: Vec<u8>) -> SetUserResponse {
    if with_state(|s| backend::api::username_exists(s, username.clone())) {
        SetUserResponse::UsernameExists
    } else {
        let user_data = User {
            // Renamed for clarity
            username,
            public_key,
        };
        with_state_mut(|s| backend::api::set_user_info(s, caller(), user_data));
        SetUserResponse::Ok
    }
}

#[query]
fn username_exists(username: String) -> bool {
    with_state(|s| backend::api::username_exists(s, username))
}

#[query]
fn who_am_i() -> WhoamiResponse {
    with_state(|s| match s.users.get(&ic_cdk::api::caller()) {
        None => WhoamiResponse::UnknownUser,
        Some(user) => WhoamiResponse::KnownUser(PublicUser {
            username: user.username.clone(),
            public_key: user.public_key.clone(),
            ic_principal: ic_cdk::api::caller(),
        }),
    })
}

#[query]
fn get_users() -> Result<Vec<PublicUser>, String> {
    match with_state(|s| backend::api::get_users(s, caller())) {
        GetUsersResponse::Users(users) => Ok(users),
        GetUsersResponse::PermissionError => Err("Permission denied.".to_string()),
    }
}

// --- Item (File/Folder) Management ---
#[update]
fn create_folder(name: String, parent_id: Option<ItemId>) -> Result<PublicItemMetadata, String> {
    with_state_mut(|s| backend::api::create_folder(s, caller(), name, parent_id))
}

#[query]
fn list_folder_contents(folder_id: Option<ItemId>) -> Result<Vec<PublicItemMetadata>, String> {
    ic_cdk::println!(
        "main.rs - list_folder_contents received folder_id: {:?}",
        folder_id
    ); // <--- ADD THIS
    with_state(|s| backend::api::list_folder_contents(s, caller(), folder_id))
}

#[update]
fn rename_item(item_id: ItemId, new_name: String) -> ItemOperationResponse {
    // Assuming backend::api::rename_item now returns Result<(), String>
    match with_state_mut(|s| backend::api::rename_item(s, caller(), item_id, new_name)) {
        Ok(()) => ItemOperationResponse::Ok, // Candid expects record for Ok variant if Ok:null
        Err(e) => ItemOperationResponse::Err(e),
    }
}

#[update]
fn delete_item(item_id: ItemId) -> ItemOperationResponse {
    // Assuming backend::api::delete_item now returns Result<(), String>
    match with_state_mut(|s| backend::api::delete_item(s, caller(), item_id)) {
        Ok(()) => ItemOperationResponse::Ok,
        Err(e) => ItemOperationResponse::Err(e),
    }
}

#[update]
fn move_item(item_id: ItemId, new_parent_id: Option<ItemId>) -> ItemOperationResponse {
    // Assuming backend::api::move_item now returns Result<(), String>
    match with_state_mut(|s| backend::api::move_item(s, caller(), item_id, new_parent_id)) {
        Ok(()) => ItemOperationResponse::Ok,
        Err(e) => ItemOperationResponse::Err(e),
    }
}

#[query]
fn get_item_owner_principal(item_id: ItemId) -> Result<Principal, String> {
    with_state(|s| {
        s.items
            .get(&item_id)
            .map(|item_meta| item_meta.owner_principal)
            .ok_or_else(|| "Item not found".to_string())
    })
}

// --- File Upload (Atomic/Direct to a Folder) ---
#[update]
fn upload_file_atomic(request: UploadFileAtomicDirectRequest) -> Result<ItemId, String> {
    // Assuming backend::api::upload_file_atomic returns ItemId or panics
    // For a more robust API, it should return Result<ItemId, String>
    Ok(with_state_mut(|s| {
        backend::api::upload_file_atomic(caller(), request, s)
    }))
}

// --- File Upload (Alias-based for pre-requested items & Chunks) ---
#[update]
fn request_file(request_name: String, parent_id: Option<ItemId>) -> Result<String, String> {
    let alias =
        with_state_mut(|s| backend::api::request_file(caller(), request_name, parent_id, s));
    // Assuming backend::api::request_file returns empty string on internal failure to create alias
    if alias.is_empty() {
        Err("Failed to create file request alias.".to_string())
    } else {
        Ok(alias)
    }
}

#[query] // Changed from update in previous .did draft, alias info should be queryable
fn get_alias_info(alias: String) -> Result<AliasInfoForUpload, GetAliasInfoError> {
    with_state(|s| backend::api::get_alias_info(s, alias))
}

#[update]
fn upload_content_to_item(request: UploadContentToItemRequest) -> DetailedUploadResponse {
    match with_state_mut(|s| {
        backend::api::upload_file(
            // This should call the refactored api::upload_file
            request.item_id,
            request.file_content,
            request.file_type,
            request.num_chunks,
            s,
        )
    }) {
        Ok(()) => DetailedUploadResponse::Ok(None),
        Err(e) => match e {
            // Map internal UploadFileError to DetailedUploadResponse
            UploadFileError::AlreadyUploaded => DetailedUploadResponse::AlreadyUploaded(None),
            UploadFileError::NotRequested => DetailedUploadResponse::NotRequested(None),
            // Add more mappings if UploadFileError gets more variants
        },
    }
}

#[update]
fn upload_chunk_continue(request: UploadChunkContinueRequest) -> DetailedUploadResponse {
    // Assuming backend::api::upload_file_continue might panic or has been refactored to return Result
    // For now, let's assume it panics on invalid state.
    with_state_mut(|s| backend::api::upload_file_continue(request, s));
    DetailedUploadResponse::Ok(None) // If it doesn't panic, assume Ok
}

// --- File Download ---
#[query]
fn download_file_chunk(item_id: ItemId, chunk_id: u64) -> DownloadChunkResponse {
    with_state(|s| backend::api::download_file(s, item_id, chunk_id, caller()))
}

// --- Sharing ---
#[update]
fn share_item(user_to_share_with: Principal, item_id: ItemId) -> ItemOperationResponse {
    match with_state_mut(|s| backend::api::share_item(s, caller(), user_to_share_with, item_id)) {
        FileSharingResponse::Ok => ItemOperationResponse::Ok,
        FileSharingResponse::PermissionError => {
            ItemOperationResponse::Err("Permission denied.".to_string())
        }
        FileSharingResponse::PendingError => ItemOperationResponse::Err(
            "Item is not in a shareable state (e.g. pending upload).".to_string(),
        ),
    }
}

#[update]
fn revoke_item_share(user_to_revoke_from: Principal, item_id: ItemId) -> ItemOperationResponse {
    match with_state_mut(|s| backend::api::revoke_share(s, caller(), user_to_revoke_from, item_id))
    {
        FileSharingResponse::Ok => ItemOperationResponse::Ok,
        FileSharingResponse::PermissionError => {
            ItemOperationResponse::Err("Permission denied or share not found.".to_string())
        }
        FileSharingResponse::PendingError => {
            ItemOperationResponse::Err("Cannot modify share for item in pending state.".to_string())
        } // Should not happen if item was shared
    }
}

#[query]
fn get_items_shared_with_me() -> Vec<PublicItemMetadata> {
    with_state(|s| backend::api::get_items_shared_with_me(s, caller()))
}

#[query]
fn get_item_sharers(item_id: ItemId) -> Result<Vec<PublicUser>, String> {
    with_state(|state_ref| {
        // state_ref is a &'a State where 'a is the lifetime of the closure
        backend::api::get_item_sharers(item_id, state_ref) // Pass the valid reference
    })
}

#[query]
fn get_item_metadata_by_id(item_id: ItemId) -> Result<PublicItemMetadata, String> {
    with_state(|s| {
        match s.items.get(&item_id) {
            Some(item) => {
                // Permission Check: Caller must own the item or have it shared with them.
                let caller = ic_cdk::api::caller();
                if item.owner_principal == caller
                    || s.item_shares
                        .get(&caller)
                        .map_or(false, |shared_ids| shared_ids.contains(&item_id))
                {
                    Ok(PublicItemMetadata {
                        id: item.id,
                        name: item.name.clone(),
                        item_type: item.item_type.clone(),
                        parent_id: item.parent_id,
                        modified_at: item.modified_at,
                        size: item.size,
                    })
                } else {
                    Err("Permission denied to access this item's metadata.".to_string())
                }
            }
            None => Err("Item not found.".to_string()),
        }
    })
}

// --- VetKD ---
// #[update]
// async fn vetkd_encrypted_key(
//     encryption_public_key: Vec<u8>,
//     item_id: Option<ItemId>,
// ) -> VetkdEncryptedKeyResponse {
//     match backend::api::vetkd_encrypted_key(encryption_public_key, item_id).await {
//         Ok(key) => VetkdEncryptedKeyResponse::Ok(key),
//         Err(msg) => VetkdEncryptedKeyResponse::Err(msg),
//     }
// }

// #[update]
// async fn vetkd_public_key() -> VetkdPublicKeyResponse {
//     match backend::api::vetkd_public_key().await {
//         Ok(key) => VetkdPublicKeyResponse::Ok(key),
//         Err(msg) => VetkdPublicKeyResponse::Err(msg),
//     }
// }

// --- "Request Group" and Template features (Legacy or to be integrated) ---
#[update]
fn multi_request_legacy(
    input: MultiRequestInputLegacy,
) -> Result<MultiRequestResponseLegacy, String> {
    Ok(with_state_mut(|s| {
        backend::api::multi_request(caller(), input, s)
    }))
}

#[query]
fn get_request_groups_legacy() -> Vec<PublicRequestGroupLegacy> {
    with_state(|s| backend::api::get_request_groups(s, caller()))
}

#[query] // Changed to query as per did
fn get_group_by_alias_legacy(
    alias: String,
) -> Result<GroupInfoForUploadResponse, GetAliasInfoError> {
    with_state(|s| backend::api::get_group_by_alias(s, alias))
}

#[query]
fn get_template_names_legacy() -> Vec<String> {
    with_state(|s| {
        backend::api::get_user_templates(s, caller())
            .into_iter()
            .map(|t| t.name)
            .collect()
    })
}

#[query]
fn get_template_legacy(name: String) -> TemplateResponseLegacy {
    match with_state(|s| backend::api::get_template(s, caller(), name)) {
        Ok(template) => TemplateResponseLegacy::Ok(template),
        Err(get_alias_info_error) => {
            // e.g. GetAliasInfoError::NotFound
            // Explicitly map the error
            match get_alias_info_error {
                GetAliasInfoError::NotFound => {
                    TemplateResponseLegacy::Err(TemplateResponseLegacyError::NotFound {})
                } // If GetAliasInfoError had other variants, you'd map them or panic/default here
            }
        }
    }
}

#[update]
fn delete_template_legacy(name: String) -> ItemOperationResponse {
    match with_state_mut(|s| backend::api::delete_template(s, caller(), name)) {
        Ok(()) => ItemOperationResponse::Ok,
        Err(_) => ItemOperationResponse::Err("Template not found or permission error.".to_string()),
    }
}

#[query]
fn get_user_templates_legacy() -> Vec<TemplateLegacy> {
    with_state(|s| backend::api::get_user_templates(s, caller()))
}

// --- Endpoint for "My Pending Requests" ---
#[query]
fn get_my_pending_requests() -> Vec<PublicItemMetadata> {
    // This calls the refactored get_requests which should now specifically list
    // file items owned by the caller that are still in a pending (alias exists, no content) state.
    with_state(|s| backend::api::get_requests(s, caller()))
}

// --- Canister Management ---
#[query]
fn get_user_canisters() -> GetUserCanistersResponse {
    backend::api::get_user_canisters()
}

#[update]
async fn register_canister(canister_id: Principal, name: String) -> RegisterCanisterResponse {
    backend::api::register_canister(canister_id, name).await
}

#[update]
async fn rename_canister(canister_id: Principal, new_name: String) -> RenameCanisterResponse {
    backend::api::rename_canister(canister_id, new_name).await
}

#[update]
async fn unregister_canister(canister_id: Principal) -> DeleteCanisterResponse {
    backend::api::unregister_canister_internal(canister_id) // This is sync in api.rs, but .did implies async. Let's keep async here.
}

// --- Simple Test Endpoint ---
#[query]
fn hello_world() -> String {
    "Hello from Docutrack!".to_string()
}

// --- Lifecycle Hooks ---
#[pre_upgrade]
fn pre_upgrade() {
    backend::pre_upgrade();
}

#[post_upgrade]
fn post_upgrade() {
    backend::post_upgrade();
}

fn main() {}

// #[cfg(test)]
// mod tests {
//     // use super::*; // Not needed if only generate_candid_file is here
//     // use candid_parser::utils::{service_equal, CandidSource};
//     // use std::path::Path;

//     #[test]
//     #[ignore]
//     fn generate_candid_file() {
//         candid::export_service!();
//         // To print to console:
//         // cargo test --manifest-path ./Cargo.toml --lib --features generate-candid -- generate_candid_file --show-output --ignored
//         println!("{}", __export_service());
//         // To write to file (uncomment and adjust path if needed):
//         // let new_candid = __export_service();
//         // std::fs::write(Path::new("./service.did"), &new_candid).expect("Write to service.did failed.");
//     }
// }
