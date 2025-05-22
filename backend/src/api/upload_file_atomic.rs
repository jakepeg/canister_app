// ic-docutrack/backend/src/api/upload_file_atomic.rs
use crate::{get_time, ItemId, ItemMetadata, ItemType, State, UploadFileAtomicDirectRequest}; // Use DTO from lib.rs
use candid::Principal;
// serde::{Deserialize, Serialize} and CandidType are not needed for the struct if it's imported

// Remove local struct definition:
// #[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct UploadFileAtomicRequest { ... }

pub fn upload_file_atomic(
    caller: Principal,
    request: UploadFileAtomicDirectRequest, // Changed to use DTO from lib.rs
    state: &mut State,
) -> ItemId {
    let item_id = state.generate_item_id();

    let new_item_metadata = ItemMetadata {
        id: item_id,
        name: request.name.clone(),
        item_type: ItemType::File,
        parent_id: request.parent_id,
        owner_principal: caller,
        created_at: get_time(),
        modified_at: get_time(),
        content_type: Some(request.file_type.clone()),
        size: Some(request.content.len() as u64),
        num_chunks: Some(request.num_chunks),
    };

    let old_value = state.items.insert(item_id, new_item_metadata);
    if old_value.is_some() {
        panic!("Overwriting an existing item should be impossible with new IDs.");
    }

    state
        .file_contents
        .insert((item_id, 0_u64), request.content);

    state.item_owners.entry(caller).or_default().push(item_id);

    item_id
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, User};
    use candid::Principal;

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
            Principal::anonymous(),
            UploadFileAtomicDirectRequest {
                // Use DTO
                name: "file_name.txt".to_string(),
                content: vec![1, 2, 3, 4, 5],
                file_type: "text/plain".to_string(),
                num_chunks: 1,
                parent_id: None,
            },
            &mut state,
        );

        let stored_item = state.items.get(&item_id).expect("Item not found");
        assert_eq!(stored_item.id, item_id);
        assert_eq!(stored_item.name, "file_name.txt");
        assert_eq!(stored_item.content_type, Some("text/plain".to_string()));
        assert_eq!(stored_item.size, Some(5));

        assert_eq!(
            state.file_contents.get(&(item_id, 0)),
            Some(vec![1u8, 2u8, 3u8, 4u8, 5u8]) // Ensure u8 type
        );
        assert!(state.file_alias_index.is_empty());
        assert_eq!(
            state.item_owners.get(&Principal::anonymous()),
            Some(&vec![item_id])
        );
    }
}
