// ic-docutrack/backend/src/api/request_file.rs
use crate::{get_time, ItemId, ItemMetadata, ItemType, State}; // Removed old File, FileContent, FileMetadata
use candid::Principal;

// user_public_key is no longer stored in ItemMetadata directly.
// The owner_principal (caller) is stored.
// use super::user_info::get_user_key;

/// Requests a file to be uploaded by generating an alias and a placeholder ItemMetadata.
pub fn request_file<S: Into<String>>(
    caller: Principal,
    request_name: S,
    parent_id: Option<ItemId>, // New: The ID of the folder where this requested file should be placed. None for root.
    state: &mut State,
) -> String {
    // TODO: verify that file alias has not been used before (though random generation makes collisions unlikely).
    let alias = state.alias_generator.next();

    let item_id = state.generate_item_id();
    let current_time = get_time();

    let item_metadata = ItemMetadata {
        id: item_id,
        name: request_name.into(),
        item_type: ItemType::File, // A request is always for a file
        parent_id,                 // Set the parent folder ID
        owner_principal: caller,   // The requester is the owner
        created_at: current_time,
        modified_at: current_time, // Initially same as created_at
        content_type: None,        // Unknown until upload
        size: None,                // Unknown until upload
        num_chunks: None,          // Unknown until upload
    };

    state.items.insert(item_id, item_metadata);
    state.file_alias_index.insert(alias.clone(), item_id);

    // The caller is the owner of this item.
    state.item_owners.entry(caller).or_default().push(item_id);

    alias
}

#[cfg(test)]
mod test {
    use crate::{api::set_user_info, get_time, ItemId, ItemMetadata, ItemType, State, User}; // Added ItemId, ItemMetadata, ItemType, get_time
    use candid::Principal;

    use super::request_file; // Import the function for testing

    #[test]
    fn requesting_a_file_updates_items_and_owners() {
        let mut state = State::default();
        let caller = Principal::anonymous();
        let parent_folder_id: Option<ItemId> = None; // Example: requesting at root

        set_user_info(
            &mut state,
            caller,
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        let alias = request_file(
            caller,
            "test_request.pdf".to_string(),
            parent_folder_id,
            &mut state,
        );
        let generated_item_id = 0; // First item_id

        // Check if item metadata is correctly stored
        let expected_item_metadata = ItemMetadata {
            id: generated_item_id,
            name: "test_request.pdf".to_string(),
            item_type: ItemType::File,
            parent_id: parent_folder_id,
            owner_principal: caller,
            created_at: get_time(), // Assuming get_time() returns a fixed value in tests
            modified_at: get_time(),
            content_type: None,
            size: None,
            num_chunks: None,
        };

        assert_eq!(
            state.items.get(&generated_item_id),
            Some(&expected_item_metadata)
        );

        // Check alias index
        assert_eq!(state.file_alias_index.get(&alias), Some(&generated_item_id));

        // Check item owners
        assert_eq!(
            state.item_owners.get(&caller),
            Some(&vec![generated_item_id])
        );
    }

    #[test]
    fn item_id_is_incrementing() {
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

        request_file(caller, "request1".to_string(), None, &mut state);
        assert_eq!(state.item_id_counter, 1); // Updated from file_count to item_id_counter

        request_file(caller, "request2".to_string(), None, &mut state);
        assert_eq!(state.item_id_counter, 2);

        assert_eq!(state.item_owners.get(&caller), Some(&vec![0, 1]));
    }
}
