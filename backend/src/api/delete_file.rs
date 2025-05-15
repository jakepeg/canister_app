// ic-docutrack/backend/src/api/delete_item.rs (renamed from delete_file.rs)
use crate::{FileSharingResponse, ItemId, ItemType, State}; // Removed FileContent. Added ItemId, ItemType, get_time.
use candid::Principal;

// Main delete function
pub fn delete_item(
    state: &mut State,
    caller: Principal,
    item_id_to_delete: ItemId,
) -> FileSharingResponse {
    // 1. Get the item to delete (cloned, so we don't hold a mutable borrow on `state.items` yet if we need to iterate it)
    let item_meta_to_delete = match state.items.get(&item_id_to_delete).cloned() {
        Some(meta) => meta,
        None => return FileSharingResponse::PermissionError, // Or a more specific "NotFound"
    };

    // 2. Check ownership
    if item_meta_to_delete.owner_principal != caller {
        return FileSharingResponse::PermissionError;
    }

    // 3. Handle based on ItemType
    match item_meta_to_delete.item_type {
        ItemType::File => {
            // Proceed with deleting the file
            delete_single_item_metadata_and_content(state, caller, item_id_to_delete);
            FileSharingResponse::Ok
        }
        ItemType::Folder => {
            // Check if the folder is empty
            let is_empty = state
                .items
                .values()
                .all(|item| item.parent_id != Some(item_id_to_delete));
            if !is_empty {
                // For now, return PermissionError. Could be a new error variant e.g., FolderNotEmpty.
                return FileSharingResponse::PermissionError; // Or specific error for non-empty folder
            }
            // Folder is empty, proceed with deleting its metadata
            delete_single_item_metadata_and_content(state, caller, item_id_to_delete);
            FileSharingResponse::Ok
        }
    }
}

// Helper function to remove item metadata, ownership, shares, and content (if applicable)
// This function assumes permission checks have already been done.
fn delete_single_item_metadata_and_content(state: &mut State, owner: Principal, item_id: ItemId) {
    // Remove item metadata
    let removed_item_meta = state.items.remove(&item_id);

    if let Some(meta) = removed_item_meta {
        // If it was a file, remove its content
        if meta.item_type == ItemType::File {
            if let Some(num_chunks) = meta.num_chunks {
                for chunk_id in 0..num_chunks {
                    state.file_contents.remove(&(item_id, chunk_id));
                }
            }
        }

        // Remove from owner's list
        if let Some(owned_items) = state.item_owners.get_mut(&owner) {
            owned_items.retain(|&id| id != item_id);
            if owned_items.is_empty() {
                state.item_owners.remove(&owner);
            }
        }

        // Remove from all share lists
        let mut shares_to_remove_user = Vec::new();
        for (shared_with_user, shared_items) in state.item_shares.iter_mut() {
            shared_items.retain(|&id| id != item_id);
            if shared_items.is_empty() {
                shares_to_remove_user.push(*shared_with_user);
            }
        }
        for user_key in shares_to_remove_user {
            state.item_shares.remove(&user_key);
        }

        // Remove from alias index if it was a pending file request
        // (Iterate and remove by value, or if alias is known, remove by key)
        let mut alias_to_remove: Option<String> = None;
        for (alias_key, &alias_item_id) in state.file_alias_index.iter() {
            if alias_item_id == item_id {
                alias_to_remove = Some(alias_key.clone());
                break;
            }
        }
        if let Some(key) = alias_to_remove {
            state.file_alias_index.remove(&key);
        }

        // Remove from request groups (if part of any)
        // This part is complex as request_groups store Vec<u64> which are ItemIds.
        // And request_groups keys are group_folder_ids.
        // If the deleted item is a file within a group:
        for request_group in state.request_groups.values_mut() {
            request_group
                .files
                .retain(|&id_in_group| id_in_group != item_id);
        }
        // If the deleted item IS a group folder itself:
        state.request_groups.remove(&item_id); // If item_id was a group_folder_id
        state.group_files.remove(&item_id); // If item_id was a group_folder_id
                                            // Also remove from group_alias_index if it's a group folder with an alias
        let mut group_alias_to_remove: Option<String> = None;
        for (alias_key, &g_id) in state.group_alias_index.iter() {
            if g_id == item_id {
                // item_id here is the group_folder_id
                group_alias_to_remove = Some(alias_key.clone());
                break;
            }
        }
        if let Some(key) = group_alias_to_remove {
            state.group_alias_index.remove(&key);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{/*request_file, upload_file,*/ set_user_info}, // These need to be updated to use ItemMetadata
        get_time, // Added ItemId, ItemMetadata, ItemType, get_time
        ItemMetadata,
        ItemType,
        User,
    };
    use candid::Principal;
    // use maplit::btreemap;

    // Helper to create a dummy item for testing delete
    fn create_dummy_item_for_delete_tests(
        state: &mut State,
        owner: Principal,
        name: &str,
        item_type: ItemType,
        parent_id: Option<ItemId>,
        num_chunks_opt: Option<u64>, // For files
    ) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        let item_metadata = ItemMetadata {
            id: item_id,
            name: name.to_string(),
            item_type: item_type.clone(), // Clone item_type here to avoid the move,
            parent_id,
            owner_principal: owner,
            created_at: current_time,
            modified_at: current_time,
            content_type: if item_type == ItemType::File {
                Some("application/octet-stream".to_string())
            } else {
                None
            },
            size: if item_type == ItemType::File {
                Some(num_chunks_opt.unwrap_or(0) * 10)
            } else {
                None
            }, // Dummy size
            num_chunks: num_chunks_opt,
        };
        state.items.insert(item_id, item_metadata);
        state.item_owners.entry(owner).or_default().push(item_id);

        // Add dummy content for files
        if item_type == ItemType::File {
            if let Some(num_chunks) = num_chunks_opt {
                for i in 0..num_chunks {
                    state.file_contents.insert((item_id, i), vec![i as u8; 10]);
                }
            }
        }
        item_id
    }

    #[test]
    fn delete_file_item_test() {
        // Renamed test
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

        // Create a file item
        let file_item_id = create_dummy_item_for_delete_tests(
            &mut state,
            caller,
            "test_file.txt",
            ItemType::File,
            None,
            Some(1),
        );
        assert!(state.items.contains_key(&file_item_id));
        assert!(state.file_contents.contains_key(&(file_item_id, 0)));

        // Delete the file item
        let result = delete_item(&mut state, caller, file_item_id);

        // Verify result and item deletion
        assert_eq!(result, FileSharingResponse::Ok);
        assert!(!state.items.contains_key(&file_item_id));
        assert!(!state.file_contents.contains_key(&(file_item_id, 0))); // Check content removed
        assert!(state
            .item_owners
            .get(&caller)
            .map_or(true, |items| !items.contains(&file_item_id)));
    }

    #[test]
    fn delete_empty_folder_item_test() {
        let mut state = State::default();
        let caller = Principal::anonymous();
        set_user_info(
            &mut state,
            caller,
            User {
                username: "FolderOwner".to_string(),
                public_key: vec![],
            },
        );

        let folder_id = create_dummy_item_for_delete_tests(
            &mut state,
            caller,
            "EmptyFolder",
            ItemType::Folder,
            None,
            None,
        );
        assert!(state.items.contains_key(&folder_id));

        let result = delete_item(&mut state, caller, folder_id);
        assert_eq!(result, FileSharingResponse::Ok);
        assert!(!state.items.contains_key(&folder_id));
    }

    #[test]
    fn delete_non_empty_folder_item_test() {
        let mut state = State::default();
        let caller = Principal::anonymous();
        set_user_info(
            &mut state,
            caller,
            User {
                username: "FolderOwner".to_string(),
                public_key: vec![],
            },
        );

        let folder_id = create_dummy_item_for_delete_tests(
            &mut state,
            caller,
            "NotEmptyFolder",
            ItemType::Folder,
            None,
            None,
        );
        let _child_file_id = create_dummy_item_for_delete_tests(
            &mut state,
            caller,
            "ChildFile.txt",
            ItemType::File,
            Some(folder_id),
            Some(1),
        );

        let result = delete_item(&mut state, caller, folder_id);
        // Expecting an error because the folder is not empty (PermissionError is used as a placeholder)
        assert_eq!(result, FileSharingResponse::PermissionError);
        assert!(state.items.contains_key(&folder_id)); // Folder should still exist
    }

    #[test]
    fn delete_item_permission_error() {
        // Renamed test
        let mut state = State::default();
        let owner = Principal::anonymous();
        let other_user = Principal::from_slice(&[0, 1, 2]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "ItemOwner".to_string(),
                public_key: vec![1, 2, 3],
            },
        );
        set_user_info(
            &mut state,
            other_user,
            User {
                username: "OtherUser".to_string(),
                public_key: vec![3, 4, 5],
            },
        );

        let item_id_to_delete = create_dummy_item_for_delete_tests(
            &mut state,
            owner,
            "owned_item.dat",
            ItemType::File,
            None,
            Some(0),
        );

        // Try to delete as 'other_user'
        let result = delete_item(&mut state, other_user, item_id_to_delete);

        // Verify permission error
        assert_eq!(result, FileSharingResponse::PermissionError);
        assert!(state.items.contains_key(&item_id_to_delete)); // Item should still exist
    }
}
