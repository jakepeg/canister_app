// ic-docutrack/backend/src/api/rename_item.rs (renamed from rename_file.rs)
use crate::{get_time, FileSharingResponse, ItemId, State}; // Added ItemId, get_time. FileSharingResponse might be renamed later.
use candid::Principal;

pub fn rename_item(
    // Renamed from rename_file
    state: &mut State,
    caller: Principal,
    item_id: ItemId, // Changed from u64 to ItemId
    new_name: String,
) -> FileSharingResponse {
    // Consider changing to Result<(), String> for more idiomatic errors
    // Check if the item exists
    let item_metadata = match state.items.get_mut(&item_id) {
        Some(meta) => meta,
        None => return FileSharingResponse::PermissionError, // Or a new "NotFound" variant
    };

    // Check if the caller owns this item
    if item_metadata.owner_principal != caller {
        return FileSharingResponse::PermissionError;
    }

    // Item exists and user has permission, update the name and modified_at timestamp
    item_metadata.name = new_name;
    item_metadata.modified_at = get_time();

    FileSharingResponse::Ok
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{/*request_file,*/ set_user_info}, // request_file needs to be updated first for this test to pass cleanly
        get_time,                               // Added ItemMetadata, ItemType, get_time
        ItemMetadata,
        ItemType,
        User,
    };
    use candid::Principal;

    // Helper to create a dummy item for testing rename
    fn create_dummy_item(
        state: &mut State,
        caller: Principal,
        name: &str,
        item_type: ItemType,
        parent_id: Option<ItemId>,
    ) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        state.items.insert(
            item_id,
            ItemMetadata {
                id: item_id,
                name: name.to_string(),
                item_type: item_type.clone(), // Clone item_type here to avoid the move
                parent_id,
                owner_principal: caller,
                created_at: current_time,
                modified_at: current_time,
                content_type: if item_type == ItemType::File {
                    Some("text/plain".to_string())
                } else {
                    None
                },
                size: if item_type == ItemType::File {
                    Some(100)
                } else {
                    None
                },
                num_chunks: if item_type == ItemType::File {
                    Some(1)
                } else {
                    None
                },
            },
        );
        state.item_owners.entry(caller).or_default().push(item_id);
        item_id
    }

    #[test]
    fn rename_item_test() {
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

        // Create a dummy file item directly for this test
        let item_id_to_rename = create_dummy_item(
            &mut state,
            caller,
            "original_name.txt",
            ItemType::File,
            None,
        );

        // Rename the item
        let result = rename_item(
            &mut state,
            caller,
            item_id_to_rename,
            "new_name.txt".to_string(),
        );

        // Verify result and new item name
        assert_eq!(result, FileSharingResponse::Ok);
        let renamed_item = state.items.get(&item_id_to_rename).unwrap();
        assert_eq!(renamed_item.name, "new_name.txt");
        assert!(
            renamed_item.modified_at > renamed_item.created_at
                || renamed_item.modified_at == renamed_item.created_at
        ); // modified_at should be updated
    }

    #[test]
    fn rename_item_permission_error() {
        // Renamed test
        let mut state = State::default();
        let owner = Principal::anonymous();
        let other_user = Principal::from_slice(&[0, 1, 2]);

        set_user_info(
            &mut state,
            owner,
            User {
                username: "JohnOwner".to_string(),
                public_key: vec![1, 2, 3],
            },
        );
        set_user_info(
            &mut state,
            other_user,
            User {
                username: "JaneOther".to_string(),
                public_key: vec![3, 4, 5],
            },
        );

        // Create a dummy file item owned by 'owner'
        let item_id_to_rename =
            create_dummy_item(&mut state, owner, "original_name.txt", ItemType::File, None);

        // Try to rename as 'other_user'
        let result = rename_item(
            &mut state,
            other_user, // Caller is not the owner
            item_id_to_rename,
            "new_name.txt".to_string(),
        );

        // Verify permission error
        assert_eq!(result, FileSharingResponse::PermissionError);
        assert_eq!(
            state.items.get(&item_id_to_rename).unwrap().name,
            "original_name.txt" // Name should not have changed
        );
    }
}
