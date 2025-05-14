// Add the new module
mod canister_management;
mod delete_file;
mod download_file;
mod get_alias_info;
mod get_group_by_alias;
mod get_request_groups;
mod get_requests;
mod get_users;
mod multi_request;
mod rename_file;
mod request_file;
mod share_file;
mod template;
mod upload_file;
mod upload_file_atomic;
mod user_info;

// Re-export functions from the new module
pub use canister_management::{
    get_user_canisters, register_canister, rename_canister, unregister_canister_internal,
};

use crate::{ItemType, State, UploadFileContinueRequest};
pub use delete_file::delete_file;
pub use download_file::download_file;
pub use get_alias_info::get_alias_info;
pub use get_group_by_alias::get_group_by_alias;
pub use get_request_groups::get_request_groups;
pub use get_requests::get_requests;
pub use get_users::get_users;
pub use multi_request::multi_request;
pub use rename_file::rename_file;
pub use request_file::request_file;
pub use share_file::{get_shared_files, revoke_share, share_file};
pub use template::{delete_template, get_template, get_user_templates}; // Added delete_template
pub use upload_file::upload_file;
pub use upload_file_atomic::{upload_file_atomic, UploadFileAtomicRequest};
pub use user_info::set_user_info;
pub use user_info::username_exists;

pub fn upload_file_continue(request: UploadFileContinueRequest, state: &mut State) {
    // Update the file's contents.
    let item_id = request.file_id; // file_id is ItemId in UploadFileContinueRequest
    let chunk_id = request.chunk_id;

    let item = match state.items.get_mut(&item_id) {
        Some(i) => i,
        None => panic!("Item with id {} not found for continuing upload.", item_id),
    };

    // Ensure it's a file
    if item.item_type != ItemType::File {
        panic!(
            "Attempted to upload chunk to a non-file item with id {}.",
            item_id
        );
    }

    // Ensure num_chunks is set (meaning upload_file or upload_file_atomic has been called for chunk 0)
    let total_num_chunks = match item.num_chunks {
        Some(n) => n,
        None => panic!(
            "Item with id {} has no num_chunks set. Upload chunk 0 first.",
            item_id
        ),
    };

    if chunk_id >= total_num_chunks {
        panic!(
            "Invalid chunk_id {} for item {} with {} total chunks.",
            chunk_id, item_id, total_num_chunks
        );
    }

    if state.file_contents.contains_key(&(item_id, chunk_id)) {
        panic!("Chunk {} already uploaded for item {}.", chunk_id, item_id);
    }

    // Calculate if this is the last chunk and total size if needed
    let chunks_uploaded = state.num_chunks_uploaded(item_id);
    let will_be_complete = chunks_uploaded + 1 == total_num_chunks;

    let mut calculated_total_size = 0;
    if will_be_complete {
        // Only calculate total size if this is the last chunk and we need it
        if state.items.get(&item_id).unwrap().size.is_none() {
            for i in 0..total_num_chunks {
                if i == chunk_id {
                    // This is the current chunk being uploaded
                    calculated_total_size += request.contents.len() as u64;
                } else if let Some(chunk_data) = state.file_contents.get(&(item_id, i)) {
                    calculated_total_size += chunk_data.len() as u64;
                }
            }
        }
    }

    // Now do the actual insert of the chunk data
    state
        .file_contents
        .insert((item_id, chunk_id), request.contents.clone());

    // Finally, update item metadata with all our pre-calculated information
    let item = state.items.get_mut(&item_id).unwrap();
    item.modified_at = crate::get_time();

    if let Some(current_size) = item.size {
        // Update existing size by adding new chunk size
        item.size = Some(current_size + request.contents.len() as u64);
    } else if will_be_complete {
        // Set the total calculated size if this completes the file and size wasn't set
        item.size = Some(calculated_total_size);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // use crate::api::user_info::get_user_key; // Not used directly in this test file anymore for chunked_upload
    use crate::{api::set_user_info, /*File, FileMetadata,*/ User}; // Keep new types
    use candid::Principal;
    // use maplit::btreemap; // Not used directly here

    #[test]
    fn chunked_upload() {
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

        // Upload the first chunk using upload_file_atomic
        let item_id = upload_file_atomic(
            caller,
            UploadFileAtomicRequest {
                name: "chunked_file.dat".to_string(),
                content: vec![1, 2, 3], // Content of chunk 0
                file_type: "application/octet-stream".to_string(),
                num_chunks: 3,
                parent_id: None,
            },
            &mut state,
        );

        // Verify first chunk and metadata
        let item_after_chunk0 = state.items.get(&item_id).unwrap();
        assert_eq!(item_after_chunk0.num_chunks, Some(3));
        assert_eq!(item_after_chunk0.size, Some(3)); // Size of first chunk
        assert_eq!(state.file_contents.get(&(item_id, 0)), Some(vec![1, 2, 3]));
        assert_eq!(state.num_chunks_uploaded(item_id), 1);

        // Upload the second chunk.
        upload_file_continue(
            UploadFileContinueRequest {
                file_id: item_id,
                chunk_id: 1,
                contents: vec![4, 5, 6],
            },
            &mut state,
        );

        let item_after_chunk1 = state.items.get(&item_id).unwrap();
        assert_eq!(item_after_chunk1.size, Some(3 + 3)); // Size updated
        assert_eq!(state.file_contents.get(&(item_id, 1)), Some(vec![4, 5, 6]));
        assert_eq!(state.num_chunks_uploaded(item_id), 2);

        // Upload the third and final chunk.
        upload_file_continue(
            UploadFileContinueRequest {
                file_id: item_id,
                chunk_id: 2,
                contents: vec![7, 8, 9, 10],
            },
            &mut state,
        );

        let item_after_chunk2 = state.items.get(&item_id).unwrap();
        // Size should be 3 (chunk0) + 3 (chunk1) + 4 (chunk2) = 10
        assert_eq!(item_after_chunk2.size, Some(10));
        assert_eq!(
            state.file_contents.get(&(item_id, 2)),
            Some(vec![7, 8, 9, 10])
        );
        assert_eq!(state.num_chunks_uploaded(item_id), 3);

        // At this point, the file is fully uploaded.
        // The item_type is File, num_chunks is Some(3), and num_chunks_uploaded is 3.
        // No explicit "FileContent::Uploaded" state to check in ItemMetadata,
        // this is derived from num_chunks and num_chunks_uploaded.
    }
}
