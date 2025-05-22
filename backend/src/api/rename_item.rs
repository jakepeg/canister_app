// ic-docutrack/backend/src/api/rename_item.rs
use crate::{get_time, ItemId, State}; // Removed FileSharingResponse
use candid::Principal;

pub fn rename_item(
    state: &mut State,
    caller: Principal,
    item_id: ItemId,
    new_name: String,
) -> Result<(), String> {
    // Changed return type
    let item_metadata = match state.items.get_mut(&item_id) {
        Some(meta) => meta,
        None => return Err("Item not found.".to_string()),
    };

    if item_metadata.owner_principal != caller {
        return Err("Permission denied: You do not own this item.".to_string());
    }

    if new_name.trim().is_empty() {
        return Err("New name cannot be empty.".to_string());
    }
    // Add other validation for name if needed (e.g., length, characters)

    item_metadata.name = new_name;
    item_metadata.modified_at = get_time();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, get_time, ItemMetadata, ItemType, User};
    use candid::Principal;

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
                item_type: item_type.clone(),
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

        let item_id_to_rename = create_dummy_item(
            &mut state,
            caller,
            "original_name.txt",
            ItemType::File,
            None,
        );
        let result = rename_item(
            &mut state,
            caller,
            item_id_to_rename,
            "new_name.txt".to_string(),
        );

        assert!(result.is_ok());
        let renamed_item = state.items.get(&item_id_to_rename).unwrap();
        assert_eq!(renamed_item.name, "new_name.txt");
    }

    #[test]
    fn rename_item_permission_error() {
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

        let item_id_to_rename =
            create_dummy_item(&mut state, owner, "original_name.txt", ItemType::File, None);
        let result = rename_item(
            &mut state,
            other_user,
            item_id_to_rename,
            "new_name.txt".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Permission denied: You do not own this item."
        );
        assert_eq!(
            state.items.get(&item_id_to_rename).unwrap().name,
            "original_name.txt"
        );
    }

    #[test]
    fn rename_item_not_found() {
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

        let result = rename_item(&mut state, caller, 999, "new_name.txt".to_string()); // 999 is a non-existent ID
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Item not found.");
    }
}
