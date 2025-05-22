// ic-docutrack/backend/src/api/delete_item.rs
use crate::{ItemId, ItemType, State}; // Removed FileSharingResponse
use candid::Principal;

pub fn delete_item(
    state: &mut State,
    caller: Principal,
    item_id_to_delete: ItemId,
) -> Result<(), String> {
    // Changed return type
    let item_meta_to_delete = match state.items.get(&item_id_to_delete).cloned() {
        Some(meta) => meta,
        None => return Err("Item not found.".to_string()),
    };

    if item_meta_to_delete.owner_principal != caller {
        return Err("Permission denied: You do not own this item.".to_string());
    }

    match item_meta_to_delete.item_type {
        ItemType::File => {
            delete_single_item_metadata_and_content(state, caller, item_id_to_delete);
            Ok(())
        }
        ItemType::Folder => {
            let is_empty = state
                .items
                .values()
                .all(|item| item.parent_id != Some(item_id_to_delete));
            if !is_empty {
                return Err("Folder is not empty. Cannot delete.".to_string());
            }
            delete_single_item_metadata_and_content(state, caller, item_id_to_delete);
            Ok(())
        }
    }
}

fn delete_single_item_metadata_and_content(state: &mut State, owner: Principal, item_id: ItemId) {
    let removed_item_meta = state.items.remove(&item_id);

    if let Some(meta) = removed_item_meta {
        if meta.item_type == ItemType::File {
            if let Some(num_chunks) = meta.num_chunks {
                for chunk_id in 0..num_chunks {
                    state.file_contents.remove(&(item_id, chunk_id));
                }
            }
        }

        if let Some(owned_items) = state.item_owners.get_mut(&owner) {
            owned_items.retain(|&id| id != item_id);
            if owned_items.is_empty() {
                state.item_owners.remove(&owner);
            }
        }

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
        for request_group in state.request_groups.values_mut() {
            request_group
                .files
                .retain(|&id_in_group| id_in_group != item_id);
        }
        // If the deleted item IS a group folder itself:
        state.request_groups.remove(&item_id);
        state.group_files.remove(&item_id);
        let mut group_alias_to_remove: Option<String> = None;
        for (alias_key, &g_id) in state.group_alias_index.iter() {
            if g_id == item_id {
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
    use crate::{api::set_user_info, get_time, ItemMetadata, ItemType, User};
    use candid::Principal;

    fn create_dummy_item_for_delete_tests(
        state: &mut State,
        owner: Principal,
        name: &str,
        item_type: ItemType,
        parent_id: Option<ItemId>,
        num_chunks_opt: Option<u64>,
    ) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        let item_metadata = ItemMetadata {
            id: item_id,
            name: name.to_string(),
            item_type: item_type.clone(),
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
            },
            num_chunks: num_chunks_opt,
        };
        state.items.insert(item_id, item_metadata);
        state.item_owners.entry(owner).or_default().push(item_id);

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

        let file_item_id = create_dummy_item_for_delete_tests(
            &mut state,
            caller,
            "test_file.txt",
            ItemType::File,
            None,
            Some(1),
        );
        assert!(state.items.contains_key(&file_item_id));
        let result = delete_item(&mut state, caller, file_item_id);

        assert!(result.is_ok());
        assert!(!state.items.contains_key(&file_item_id));
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

        let result = delete_item(&mut state, caller, folder_id);
        assert!(result.is_ok());
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
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Folder is not empty. Cannot delete.");
        assert!(state.items.contains_key(&folder_id));
    }

    #[test]
    fn delete_item_permission_error() {
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

        let result = delete_item(&mut state, other_user, item_id_to_delete);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Permission denied: You do not own this item."
        );
        assert!(state.items.contains_key(&item_id_to_delete));
    }

    #[test]
    fn delete_item_not_found() {
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

        let result = delete_item(&mut state, caller, 999); // Non-existent ID
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item not found.");
    }
}
