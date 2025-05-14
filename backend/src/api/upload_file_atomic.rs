use crate::{get_time, ItemId, ItemMetadata, ItemType, State};
use candid::CandidType;
use candid::Principal;
use serde::{Deserialize, Serialize};
// Not used as we aren't storing encrypted_keys while sharing anymore
// use std::collections::BTreeMap;

// use super::user_info::get_user_key;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UploadFileAtomicRequest {
    pub name: String,
    pub content: Vec<u8>,
    // pub owner_key: Vec<u8>,
    pub file_type: String,
    pub num_chunks: u64,
    pub parent_id: Option<ItemId>,
}

pub fn upload_file_atomic(
    caller: Principal,
    request: UploadFileAtomicRequest,
    state: &mut State,
) -> ItemId {
    // let file_id = state.generate_file_id();
    let item_id = state.generate_item_id();

    // Create new ItemMetadata
    let new_item_metadata = ItemMetadata {
        id: item_id,
        name: request.name.clone(), // Clone to avoid move error if request.name is used later
        item_type: ItemType::File,
        parent_id: request.parent_id,
        owner_principal: caller,
        created_at: get_time(),
        modified_at: get_time(),
        content_type: Some(request.file_type.clone()), // Clone for the same reason
        // Size of the first chunk. If num_chunks > 1, this will be updated later or calculated.
        // For atomic upload where num_chunks == 1, this is the total size.
        size: Some(request.content.len() as u64),
        num_chunks: Some(request.num_chunks),
    };

    // Add item metadata to state
    let old_value = state.items.insert(item_id, new_item_metadata);
    if old_value.is_some() {
        // This should ideally not happen with auto-incrementing IDs
        panic!("Overwriting an existing item should be impossible with new IDs.");
    }

    // Add file contents (first chunk) to stable store.
    // request.content is moved here.
    state
        .file_contents
        .insert((item_id, 0_u64), request.content);

    // Add the caller as the owner of this item.
    state.item_owners.entry(caller).or_default().push(item_id);

    item_id
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, /*File, FileMetadata,*/ User}; // Commented out old types

    #[test]
    fn stores_file_in_state() {
        let mut state = State::default();

        set_user_info(
            &mut state,
            Principal::anonymous(),
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        let item_id = upload_file_atomic(
            // Store the returned item_id
            Principal::anonymous(),
            UploadFileAtomicRequest {
                name: "file_name.txt".to_string(),
                content: vec![1, 2, 3, 4, 5],
                file_type: "text/plain".to_string(),
                num_chunks: 1,
                parent_id: None, // Example: uploading to root
            },
            &mut state,
        );

        // Verify the item is stored in state.items
        let stored_item = state.items.get(&item_id).expect("Item not found");
        assert_eq!(stored_item.id, item_id);
        assert_eq!(stored_item.name, "file_name.txt");
        assert_eq!(stored_item.item_type, ItemType::File);
        assert_eq!(stored_item.parent_id, None);
        assert_eq!(stored_item.owner_principal, Principal::anonymous());
        assert_eq!(stored_item.content_type, Some("text/plain".to_string()));
        assert_eq!(stored_item.size, Some(5));
        assert_eq!(stored_item.num_chunks, Some(1));

        // Verify file contents
        assert_eq!(
            state.file_contents.get(&(item_id, 0)),
            Some(vec![1, 2, 3, 4, 5])
        );

        // The alias index should be empty for atomic uploads unless an alias was explicitly created.
        // This test doesn't involve aliases, so it should remain empty.
        assert!(state.file_alias_index.is_empty());

        // Verify owners
        assert_eq!(
            state.item_owners.get(&Principal::anonymous()),
            Some(&vec![item_id])
        );
    }
}
