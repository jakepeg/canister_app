use crate::{get_time, File, FileContent, FileMetadata, ItemId, ItemMetadata, ItemType, State};
use candid::CandidType;
use candid::Principal;
use serde::{Deserialize, Serialize};
// Not used as we aren't storing encrypted_keys while sharing anymore
// use std::collections::BTreeMap;

use super::user_info::get_user_key;

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
) -> u64 {
    // let file_id = state.generate_file_id();
    let item_id = state.generate_item_id();

    let content = if request.num_chunks == 1 {
        // File is uploaded in one chunk.
        FileContent::Uploaded {
            num_chunks: request.num_chunks,
            file_type: request.file_type,
            // owner_key: request.owner_key,
            // Remove shared_keys as it's no longer needed
            // shared_keys: BTreeMap::new(),
        }
    } else {
        // File will be uploaded in multiple chunks.
        FileContent::PartiallyUploaded {
            num_chunks: request.num_chunks,
            file_type: request.file_type,
            // owner_key: request.owner_key,
            // Remove shared_keys as it's no longer needed
            // shared_keys: BTreeMap::new(),
        }
    };

    // Add file contents to stable store.
    let chunk_id = 0;
    state
        .file_contents
        .insert((item_id, chunk_id), request.content);

    let old_value = state.items.insert(
        item_id,
        ItemMetadata {
            id: item_id,
            name: request.name.clone(), // From request
            item_type: ItemType::File,
            parent_id: request.parent_id, // From request
            owner_principal: caller,
            created_at: get_time(),
            modified_at: get_time(),
            content_type: Some(request.file_type.clone()),
            size: Some(request.content.len() as u64), // Initial size, can be updated for multi-chunk
            num_chunks: Some(request.num_chunks),
        },
    );

    if old_value.is_some() {
        panic!("Overwriting an existing file should be impossible.");
    }

    // Add the caller as the owner of this file.
    state
        .item_owners
        .entry(caller)
        .or_insert_with(Vec::new)
        .push(item_id);

    item_id
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, File, FileMetadata, User};
    use maplit::btreemap;

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

        // Request a file.
        upload_file_atomic(
            Principal::anonymous(),
            UploadFileAtomicRequest {
                num_chunks: 1,
                name: "file_name".to_string(),
                content: vec![1, 2, 3],
                // owner_key: vec![1, 2, 3],
                file_type: "image/jpeg".to_string(),
            },
            &mut state,
        );

        // The file is stored in the state.
        assert_eq!(
            state.file_data,
            btreemap! {
                0 => File {
                    metadata: FileMetadata {
                        file_name: "file_name".to_string(),
                        user_public_key: get_user_key(&state, Principal::anonymous()),
                        requester_principal: Principal::anonymous(),
                        requested_at: get_time(),
                        uploaded_at: Some(get_time()),
                    },
                    content: FileContent::Uploaded {
                        file_type: "image/jpeg".to_string(),
                        // owner_key: vec![1,2,3],
                        // Remove shared_keys as it's no longer needed
                        // shared_keys: BTreeMap::new(),
                        num_chunks: 1,
                    }
                }
            }
        );
        assert_eq!(state.file_contents.get(&(0, 0)), Some(vec![1, 2, 3]));

        // The alias index is empty.
        assert!(state.file_alias_index.is_empty());

        // Owners are updated.
        // TODO: test this logic with the get_files endpoint.
        assert_eq!(
            state.file_owners,
            btreemap! {
                Principal::anonymous() => vec![0],
            }
        );
    }
}
