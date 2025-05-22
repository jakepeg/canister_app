// ic-docutrack/backend/src/api.rs
mod canister_management;
mod create_folder;
mod delete_item; // Renamed
mod download_file;
mod get_alias_info;
mod get_group_by_alias;
mod get_request_groups;
mod get_requests;
mod get_users;
mod list_folder_contents;
mod move_item;
mod multi_request;
mod rename_item; // Renamed
mod request_file;
mod share_item;
mod template;
mod upload_file;
mod upload_file_atomic;
mod user_info;

use crate::{
    get_time, ItemId, ItemMetadata, ItemType, State, UploadChunkContinueRequest,
    UploadFileAtomicDirectRequest, UploadFileError,
};
pub use canister_management::{
    get_user_canisters, register_canister, rename_canister, unregister_canister_internal,
};
pub use create_folder::create_folder;
pub use delete_item::delete_item; // Updated
pub use download_file::download_file;
pub use get_alias_info::get_alias_info;
pub use get_group_by_alias::get_group_by_alias;
pub use get_request_groups::get_request_groups;
pub use get_requests::get_requests;
pub use get_users::get_users;
pub use list_folder_contents::list_folder_contents;
pub use move_item::move_item;
pub use multi_request::multi_request;
pub use rename_item::rename_item; // Updated
pub use request_file::request_file;
pub use share_item::{get_item_sharers, get_items_shared_with_me, revoke_share, share_item};
pub use template::{delete_template, get_template, get_user_templates};
pub use upload_file::upload_file;
pub use upload_file_atomic::upload_file_atomic;
pub use user_info::{set_user_info, username_exists};

pub fn upload_content_to_item(
    item_id: ItemId,
    contents: Vec<u8>,
    file_type: String,
    num_chunks: u64,
    state: &mut State,
) -> Result<(), UploadFileError> {
    crate::api::upload_file::upload_file(item_id, contents, file_type, num_chunks, state)
}

pub fn upload_file_continue(request: UploadChunkContinueRequest, state: &mut State) {
    let item_id = request.item_id;
    let chunk_id = request.chunk_id;

    let total_num_chunks = {
        let item_m_ro = match state.items.get(&item_id) {
            Some(i) => i,
            None => panic!("Item with id {} not found for continuing upload.", item_id),
        };
        if item_m_ro.item_type != ItemType::File {
            panic!(
                "Attempted to upload chunk to a non-file item with id {}.",
                item_id
            );
        }
        match item_m_ro.num_chunks {
            Some(n) => n,
            None => panic!(
                "Item with id {} has no num_chunks set. Upload chunk 0 first.",
                item_id
            ),
        }
    };

    if chunk_id >= total_num_chunks {
        panic!(
            "Invalid chunk_id {} for item {} with {} total chunks.",
            chunk_id, item_id, total_num_chunks
        );
    }
    if state.file_contents.contains_key(&(item_id, chunk_id)) {
        // Allow re-upload
    }

    let current_chunk_size = request.contents.len() as u64;
    state
        .file_contents
        .insert((item_id, chunk_id), request.contents.clone());

    let num_chunks_now_uploaded = state.num_chunks_uploaded(item_id);

    let final_size_to_set: Option<u64>;
    if num_chunks_now_uploaded == total_num_chunks {
        let mut calculated_total_size: u64 = 0;
        for i in 0..total_num_chunks {
            if let Some(chunk_data) = state.file_contents.get(&(item_id, i)) {
                calculated_total_size += chunk_data.len() as u64;
            } else {
                panic!("Inconsistency: All chunks reported uploaded for item {} but chunk {} data is missing.", item_id, i);
            }
        }
        final_size_to_set = Some(calculated_total_size);
    } else {
        let current_size_opt = state.items.get(&item_id).unwrap().size;
        final_size_to_set = Some(current_size_opt.unwrap_or(0) + current_chunk_size);
    }

    let item_metadata_to_update = state.items.get_mut(&item_id).unwrap();
    item_metadata_to_update.modified_at = get_time();
    if let Some(new_size) = final_size_to_set {
        item_metadata_to_update.size = Some(new_size);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, User};
    use candid::Principal;

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

        let item_id = upload_file_atomic(
            caller,
            UploadFileAtomicDirectRequest {
                name: "chunked_file.dat".to_string(),
                content: vec![1u8, 2u8, 3u8],
                file_type: "application/octet-stream".to_string(),
                num_chunks: 3,
                parent_id: None,
            },
            &mut state,
        );

        upload_file_continue(
            UploadChunkContinueRequest {
                item_id: item_id,
                chunk_id: 1,
                contents: vec![4u8, 5u8, 6u8],
            },
            &mut state,
        );
        upload_file_continue(
            UploadChunkContinueRequest {
                item_id: item_id,
                chunk_id: 2,
                contents: vec![7u8, 8u8, 9u8, 10u8],
            },
            &mut state,
        );

        let item_after_chunk2 = state.items.get(&item_id).unwrap();
        assert_eq!(item_after_chunk2.size, Some(10));
        assert_eq!(state.num_chunks_uploaded(item_id), 3);
    }
}
