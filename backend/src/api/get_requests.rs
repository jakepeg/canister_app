// ic-docutrack/backend/src/api/get_requests.rs
use crate::{
    // FileContent, // Old
    FileStatus, // Old, needs re-evaluation or replacement
    ItemId,
    ItemType, // New types
    // PublicFileMetadata, // Old, use PublicItemMetadata or a new DTO
    PublicItemMetadata, // Using this for now, might need a more specific DTO for "requests"
    PublicUser,
    State,
};
use candid::Principal;

// This function's purpose needs to be clearly defined.
// If it's "list all items owned by me", then list_folder_contents(None) is better.
// If it's "list my pending file requests (via alias)", then it needs different logic.
// Assuming the latter for now, as "requests" often implies alias-based.
pub fn get_requests(state: &State, caller: Principal) -> Vec<PublicItemMetadata> {
    // Returning PublicItemMetadata for now
    let mut result_items = Vec::new();

    // Iterate through items owned by the caller
    if let Some(owned_item_ids) = state.item_owners.get(&caller) {
        for item_id in owned_item_ids {
            if let Some(item) = state.items.get(item_id) {
                // A "request" is a file that is still pending an upload via an alias.
                // This means it should have an entry in file_alias_index and typically no content_type yet.
                let is_pending_alias_request =
                    state.file_alias_index.values().any(|&id| id == *item_id)
                        && item.item_type == ItemType::File
                        && item.content_type.is_none(); // Or more robust status check

                if is_pending_alias_request {
                    // Convert ItemMetadata to PublicItemMetadata
                    // The old PublicFileMetadata had group_name, group_alias, file_status, shared_with.
                    // We need to map these concepts to the new structure if they are still relevant for "requests".
                    // For a simple pending request, some fields might be default or derived.

                    // Deriving a simplified status for PublicItemMetadata display
                    // let display_status = if item.content_type.is_none() {
                    //     // Simplified: if no content_type, assume pending.
                    //     // More complex status logic might be needed if FileStatus enum is kept.
                    // }

                    let public_item = PublicItemMetadata {
                        id: item.id,
                        name: item.name.clone(),
                        item_type: item.item_type.clone(),
                        parent_id: item.parent_id,
                        modified_at: item.modified_at,
                        size: item.size, // Will be None for pending requests
                                         // owner_principal: item.owner_principal, // Add if frontend needs it
                    };
                    result_items.push(public_item);
                }
            }
        }
    }
    result_items
}

// This get_allowed_users function is for sharing. It needs to use item_shares.
pub fn get_allowed_users(state: &State, item_id: ItemId) -> Vec<PublicUser> {
    // Changed to ItemId
    state
        .item_shares // Changed from file_shares
        .iter()
        .filter_map(|(user_principal, shared_item_ids)| {
            if shared_item_ids.contains(&item_id) {
                state.users.get(user_principal).map(|user| PublicUser {
                    username: user.username.clone(),
                    public_key: user.public_key.clone(),
                    ic_principal: *user_principal,
                })
            } else {
                None
            }
        })
        .collect()
}

// The `FileStatus` enum is problematic as it's from the old system.
// Status should ideally be derived from ItemMetadata.
// This function needs significant rework or removal.
// For now, I'll provide a version that tries to map, but it's not ideal.
pub fn get_item_status(state: &State, item_id: ItemId) -> FileStatus {
    // Renamed, but FileStatus is old
    match state.items.get(&item_id) {
        Some(item) => {
            if item.item_type == ItemType::Folder {
                // Folders don't have the same status types.
                // What should be returned here? For now, let's say "Uploaded" if it exists.
                // This is a placeholder and needs proper definition.
                return FileStatus::Uploaded {
                    uploaded_at: item.modified_at,
                };
            }

            // For Files:
            if item.content_type.is_none() {
                // If no content_type, it's likely a pending alias request.
                // We need to find the alias.
                let alias = state
                    .file_alias_index
                    .iter()
                    .find_map(|(a, id)| {
                        if *id == item_id {
                            Some(a.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "unknown_alias".to_string()); // Fallback, should ideally not happen

                FileStatus::Pending {
                    alias,
                    requested_at: item.created_at,
                }
            } else {
                // Has content_type, so it's at least partially uploaded.
                let total_chunks_expected = item.num_chunks.unwrap_or(1); // Assume 1 if not set (though it should be)
                let chunks_uploaded_count = state.num_chunks_uploaded(item_id);

                if chunks_uploaded_count >= total_chunks_expected {
                    FileStatus::Uploaded {
                        uploaded_at: item.modified_at,
                    }
                } else {
                    FileStatus::PartiallyUploaded
                }
            }
        }
        None => {
            // Item not found, what status to return? This case should be handled by the caller.
            // For now, let's return a generic pending status, but this is not robust.
            FileStatus::Pending {
                alias: "not_found".to_string(),
                requested_at: 0,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{request_file, set_user_info}, // request_file now takes parent_id
        get_time,
        ItemMetadata, // Added ItemType, ItemMetadata
                      // PublicFileMetadata, // This is the old struct for tests, might need to update tests to use PublicItemMetadata
        ItemType,
        User,
    };
    use candid::Principal;

    #[test]
    fn get_files_test() {
        // Test name might need update, e.g., get_pending_requests_test
        let mut state = State::default();
        let caller = Principal::anonymous();
        set_user_info(
            &mut state,
            caller,
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        // Request files (these will be pending)
        let alias1 = request_file(caller, "request1.txt".to_string(), None, &mut state);
        let _alias2 = request_file(caller, "request2.pdf".to_string(), None, &mut state);

        // Create a folder (not a pending request, so shouldn't appear in `get_requests` output)
        let folder_id = state.generate_item_id();
        state.items.insert(
            folder_id,
            ItemMetadata {
                id: folder_id,
                name: "MyFolder".to_string(),
                item_type: ItemType::Folder,
                parent_id: None,
                owner_principal: caller,
                created_at: get_time(),
                modified_at: get_time(),
                content_type: None,
                size: None,
                num_chunks: None,
            },
        );
        state.item_owners.entry(caller).or_default().push(folder_id);

        let requests = get_requests(&state, caller);
        assert_eq!(requests.len(), 2); // Should only list the two pending files

        // Check details of the first pending request
        let req1 = requests.iter().find(|r| r.name == "request1.txt").unwrap();
        assert_eq!(req1.id, 0); // Assuming first requested item gets ID 0
        assert_eq!(req1.item_type, ItemType::File);
        assert_eq!(req1.parent_id, None);
        assert!(req1.size.is_none());

        // To properly test `FileStatus::Pending { alias, ... }`, `get_requests` would need to return a DTO
        // that includes the alias or status directly, or the test would need to call `get_item_status`.
        // The current `PublicItemMetadata` doesn't have `alias` or explicit `FileStatus`.

        // Example of checking status separately (if FileStatus is still used)
        let status_req1 = get_item_status(&state, req1.id);
        if let FileStatus::Pending { alias, .. } = status_req1 {
            assert_eq!(alias, alias1);
        } else {
            panic!("Expected Pending status for request1.txt");
        }
    }
}
