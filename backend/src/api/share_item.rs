// ic-docutrack/backend/src/api/share_item.rs (updated)
use crate::{
    get_time,
    FileSharingResponse, // This internal response is mapped by main.rs
    ItemId,
    ItemType,
    PublicItemMetadata,
    PublicUser, // For get_item_sharers
    State,
};
use candid::Principal;

pub fn share_item(
    state: &mut State,
    caller: Principal,
    sharing_with: Principal,
    item_id: ItemId,
) -> FileSharingResponse {
    // 1. Get item to check ownership and state
    let item_can_be_shared = {
        // Renamed variable for clarity
        let item = match state.items.get(&item_id) {
            Some(i) => i,
            None => return FileSharingResponse::PermissionError, // Item not found
        };

        if item.owner_principal != caller {
            return FileSharingResponse::PermissionError; // Caller is not owner
        }

        // For files, check if fully uploaded. Folders can always be shared if owned.
        if item.item_type == ItemType::File {
            let total_chunks_expected = item.num_chunks.unwrap_or(0);
            if total_chunks_expected == 0 || item.content_type.is_none() {
                // Not yet uploaded (pending alias or no content at all)
                return FileSharingResponse::PendingError;
            }
            let chunks_uploaded = state.num_chunks_uploaded(item_id);
            if chunks_uploaded < total_chunks_expected {
                // Partially uploaded
                return FileSharingResponse::PendingError;
            }
        }
        true // Item can be shared
    };

    if item_can_be_shared {
        let item_shares_for_user = state.item_shares.entry(sharing_with).or_default();
        if !item_shares_for_user.contains(&item_id) {
            item_shares_for_user.push(item_id);
            if let Some(item_meta) = state.items.get_mut(&item_id) {
                item_meta.modified_at = get_time();
            }
        }
        FileSharingResponse::Ok
    } else {
        // This path should ideally not be reached if checks above are correct,
        // but as a fallback.
        FileSharingResponse::PermissionError
    }
}

// Renamed from can_share to be more specific
fn can_caller_modify_item_shares(state: &State, user: Principal, item_id: ItemId) -> bool {
    state
        .items
        .get(&item_id)
        .map_or(false, |item| item.owner_principal == user)
}

pub fn revoke_share(
    state: &mut State,
    caller: Principal,
    sharing_with: Principal,
    item_id: ItemId,
) -> FileSharingResponse {
    if !can_caller_modify_item_shares(state, caller, item_id) {
        return FileSharingResponse::PermissionError;
    }

    let mut user_had_no_shares_at_all = false;
    let mut item_was_not_shared_with_this_user = false;

    let should_remove_user_entry = match state.item_shares.get_mut(&sharing_with) {
        Some(shared_items_for_user) => {
            let initial_len = shared_items_for_user.len();
            shared_items_for_user.retain(|&id| id != item_id);
            let item_actually_removed = shared_items_for_user.len() < initial_len;

            if !item_actually_removed {
                item_was_not_shared_with_this_user = true;
            } else {
                // Update modified_at timestamp of the item
                if let Some(item_meta) = state.items.get_mut(&item_id) {
                    item_meta.modified_at = get_time();
                }
            }
            shared_items_for_user.is_empty()
        }
        None => {
            user_had_no_shares_at_all = true;
            false // No entry to remove
        }
    };

    if user_had_no_shares_at_all || item_was_not_shared_with_this_user {
        return FileSharingResponse::PermissionError; // Or a more specific error like "ShareNotFound"
    }

    if should_remove_user_entry {
        state.item_shares.remove(&sharing_with);
    }

    FileSharingResponse::Ok
}

pub fn get_items_shared_with_me(state: &State, caller: Principal) -> Vec<PublicItemMetadata> {
    match state.item_shares.get(&caller) {
        None => vec![],
        Some(item_ids) => item_ids
            .iter()
            .filter_map(|item_id| {
                state
                    .items
                    .get(item_id)
                    .map(|item_meta| PublicItemMetadata {
                        id: item_meta.id,
                        name: item_meta.name.clone(),
                        item_type: item_meta.item_type.clone(),
                        parent_id: item_meta.parent_id,
                        modified_at: item_meta.modified_at,
                        size: item_meta.size,
                    })
            })
            .collect(),
    }
}

// New function as requested by main.rs
pub fn get_item_sharers(item_id: ItemId, state: &State) -> Result<Vec<PublicUser>, String> {
    // First, check if the item exists.
    if !state.items.contains_key(&item_id) {
        return Err("Item not found.".to_string());
    }
    // (Optionally, add permission check: only owner or a sharer can see other sharers)
    // For now, let's assume if item exists, anyone can query its sharers for simplicity.

    let mut sharers = Vec::new();
    for (user_principal, shared_item_ids) in state.item_shares.iter() {
        if shared_item_ids.contains(&item_id) {
            if let Some(user_info) = state.users.get(user_principal) {
                sharers.push(PublicUser {
                    username: user_info.username.clone(),
                    public_key: user_info.public_key.clone(),
                    ic_principal: *user_principal,
                });
            }
            // If user_info is not found, it's an inconsistency, but we can skip.
        }
    }
    Ok(sharers)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, get_time, ItemId, ItemMetadata, ItemType, State, User};
    use candid::Principal;

    fn create_dummy_shareable_item(
        state: &mut State,
        owner: Principal,
        name: &str,
        item_type: ItemType,
        is_fully_uploaded: bool,
    ) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        let (content_type, size, num_chunks) = if item_type == ItemType::File && is_fully_uploaded {
            (
                Some("application/pdf".to_string()),
                Some(1024_u64),
                Some(1_u64),
            )
        } else if item_type == ItemType::File && !is_fully_uploaded {
            (
                Some("application/pdf".to_string()),
                Some(512_u64), // Example: 1 of 2 chunks uploaded
                Some(2_u64),
            )
        } else {
            (None, None, None)
        };

        state.items.insert(
            item_id,
            ItemMetadata {
                id: item_id,
                name: name.to_string(),
                item_type: item_type.clone(),
                parent_id: None,
                owner_principal: owner,
                created_at: current_time,
                modified_at: current_time,
                content_type,
                size,
                num_chunks,
            },
        );
        state.item_owners.entry(owner).or_default().push(item_id);
        if item_type == ItemType::File && is_fully_uploaded {
            state.file_contents.insert((item_id, 0), vec![0; 1024]);
        } else if item_type == ItemType::File && !is_fully_uploaded && num_chunks == Some(2) {
            state.file_contents.insert((item_id, 0), vec![0; 512]); // Chunk 0 of 2
        }
        item_id
    }

    #[test]
    fn share_item_test() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        let share_recipient = Principal::from_slice(&[0, 1, 2]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            share_recipient,
            User {
                username: "Recipient".to_string(),
                public_key: vec![],
            },
        );

        let file_item_id =
            create_dummy_shareable_item(&mut state, owner, "share_me.pdf", ItemType::File, true);
        let folder_item_id =
            create_dummy_shareable_item(&mut state, owner, "SharedFolder", ItemType::Folder, true);

        let share_file_result = share_item(&mut state, owner, share_recipient, file_item_id);
        assert_eq!(share_file_result, FileSharingResponse::Ok);
        assert!(state
            .item_shares
            .get(&share_recipient)
            .unwrap()
            .contains(&file_item_id));

        let share_folder_result = share_item(&mut state, owner, share_recipient, folder_item_id);
        assert_eq!(share_folder_result, FileSharingResponse::Ok);
        assert!(state
            .item_shares
            .get(&share_recipient)
            .unwrap()
            .contains(&folder_item_id));

        let shared_with_recipient = get_items_shared_with_me(&state, share_recipient);
        assert_eq!(shared_with_recipient.len(), 2);
    }

    #[test]
    fn get_item_sharers_test() {
        let mut state = State::default();
        let owner = Principal::from_slice(&[1]);
        let recipient1 = Principal::from_slice(&[2]);
        let recipient2 = Principal::from_slice(&[3]);
        let non_recipient = Principal::from_slice(&[4]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".into(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            recipient1,
            User {
                username: "Rec1".into(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            recipient2,
            User {
                username: "Rec2".into(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            non_recipient,
            User {
                username: "NonRec".into(),
                public_key: vec![],
            },
        );

        let item_id =
            create_dummy_shareable_item(&mut state, owner, "item.dat", ItemType::File, true);

        share_item(&mut state, owner, recipient1, item_id);
        share_item(&mut state, owner, recipient2, item_id);

        let sharers_result = get_item_sharers(item_id, &state);
        assert!(sharers_result.is_ok());
        let sharers = sharers_result.unwrap();
        assert_eq!(sharers.len(), 2);
        assert!(sharers
            .iter()
            .any(|u| u.ic_principal == recipient1 && u.username == "Rec1"));
        assert!(sharers
            .iter()
            .any(|u| u.ic_principal == recipient2 && u.username == "Rec2"));
        assert!(!sharers.iter().any(|u| u.ic_principal == owner)); // Owner is not in sharers list
        assert!(!sharers.iter().any(|u| u.ic_principal == non_recipient));

        let non_existent_item_sharers = get_item_sharers(999, &state); // Non-existent item
        assert!(non_existent_item_sharers.is_err());
        assert_eq!(non_existent_item_sharers.unwrap_err(), "Item not found.");
    }

    // ... other tests from share_file.rs should be here and pass ...
    #[test]
    fn share_item_permission_error_not_owner() {
        let mut state = State::default();
        let owner = Principal::from_slice(&[1]);
        let non_owner_caller = Principal::from_slice(&[2]);
        let share_recipient = Principal::from_slice(&[3]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            non_owner_caller,
            User {
                username: "NonOwner".to_string(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            share_recipient,
            User {
                username: "Recipient".to_string(),
                public_key: vec![],
            },
        );

        let item_id =
            create_dummy_shareable_item(&mut state, owner, "secret.txt", ItemType::File, true);

        let result = share_item(&mut state, non_owner_caller, share_recipient, item_id);
        assert_eq!(result, FileSharingResponse::PermissionError);
    }

    #[test]
    fn share_pending_file_item_error() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        let recipient = Principal::from_slice(&[1]);
        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".into(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            recipient,
            User {
                username: "Recipient".into(),
                public_key: vec![],
            },
        );

        let item_id = state.generate_item_id();
        state.items.insert(
            item_id,
            ItemMetadata {
                id: item_id,
                name: "pending.dat".to_string(),
                item_type: ItemType::File,
                parent_id: None,
                owner_principal: owner,
                created_at: get_time(),
                modified_at: get_time(),
                content_type: None,
                size: None,
                num_chunks: None,
            },
        );
        state.item_owners.entry(owner).or_default().push(item_id);

        let result = share_item(&mut state, owner, recipient, item_id);
        assert_eq!(result, FileSharingResponse::PendingError);
    }

    #[test]
    fn revoke_share_test() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        let share_recipient = Principal::from_slice(&[0, 1, 2]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        set_user_info(
            &mut state,
            share_recipient,
            User {
                username: "Recipient".to_string(),
                public_key: vec![],
            },
        );

        let item_id =
            create_dummy_shareable_item(&mut state, owner, "revoke_me.txt", ItemType::File, true);
        share_item(&mut state, owner, share_recipient, item_id);
        assert!(state
            .item_shares
            .get(&share_recipient)
            .unwrap()
            .contains(&item_id));

        let revoke_result = revoke_share(&mut state, owner, share_recipient, item_id);
        assert_eq!(revoke_result, FileSharingResponse::Ok);
        assert!(state
            .item_shares
            .get(&share_recipient)
            .map_or(true, |items| !items.contains(&item_id)));
    }
}
