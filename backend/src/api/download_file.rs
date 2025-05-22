// ic-docutrack/backend/src/api/download_file.rs
use crate::{DownloadChunkResponse, FoundFileChunk, ItemId, ItemType, State}; // Changed FileData -> FoundFileChunk, FileDownloadResponse -> DownloadChunkResponse
use candid::Principal;

// Helper for owned item download
// The return type and internal logic needs to map to DownloadChunkResponse and FoundFileChunk
fn get_owned_item_data(s: &State, item_id: ItemId, chunk_id: u64) -> DownloadChunkResponse {
    let item = match s.items.get(&item_id) {
        Some(i) => i,
        None => return DownloadChunkResponse::NotFoundItem, // Item itself not found
    };

    if item.item_type == ItemType::Folder {
        return DownloadChunkResponse::NotAFile;
    }

    let content_type = match &item.content_type {
        Some(ct) => ct.clone(),
        None => return DownloadChunkResponse::NotUploadedFile,
    };
    let num_chunks_total = match item.num_chunks {
        Some(nc) => nc,
        None => return DownloadChunkResponse::NotUploadedFile,
    };

    if s.num_chunks_uploaded(item_id) < num_chunks_total {
        return DownloadChunkResponse::NotUploadedFile;
    }

    match s.file_contents.get(&(item_id, chunk_id)) {
        Some(contents) => DownloadChunkResponse::FoundFileChunk(FoundFileChunk {
            // Updated struct name
            contents: contents.clone(),
            file_type: content_type,
            num_chunks: num_chunks_total,
        }),
        None => DownloadChunkResponse::ChunkNotFound, // Specific chunk not found
    }
}

// Helper for shared item download
fn get_shared_item_data(
    s: &State,
    item_id: ItemId,
    chunk_id: u64,
    _user: Principal,
) -> DownloadChunkResponse {
    get_owned_item_data(s, item_id, chunk_id) // Logic is similar for now
}

pub fn download_file(
    s: &State,
    item_id: ItemId,
    chunk_id: u64,
    caller: Principal,
) -> DownloadChunkResponse {
    let item = match s.items.get(&item_id) {
        Some(meta) => meta,
        None => return DownloadChunkResponse::NotFoundItem,
    };

    let is_owner = item.owner_principal == caller;
    let is_shared = is_item_shared_with_me(s, item_id, caller);

    if !is_owner && !is_shared {
        return DownloadChunkResponse::PermissionError;
    }

    if item.item_type == ItemType::Folder {
        return DownloadChunkResponse::NotAFile;
    }

    if item.content_type.is_none() || item.num_chunks.is_none() {
        return DownloadChunkResponse::NotUploadedFile;
    }
    let total_chunks_expected = item.num_chunks.unwrap();
    if s.num_chunks_uploaded(item_id) < total_chunks_expected {
        return DownloadChunkResponse::NotUploadedFile;
    }

    if chunk_id >= total_chunks_expected {
        return DownloadChunkResponse::ChunkNotFound;
    }

    match s.file_contents.get(&(item_id, chunk_id)) {
        Some(contents) => DownloadChunkResponse::FoundFileChunk(FoundFileChunk {
            // Updated struct name
            contents: contents.clone(),
            file_type: item.content_type.as_ref().unwrap().clone(),
            num_chunks: total_chunks_expected,
        }),
        None => DownloadChunkResponse::ChunkNotFound, // Should be caught by chunk_id >= total_chunks_expected, but good to have
    }
}

fn is_item_shared_with_me(s: &State, item_id: ItemId, caller: Principal) -> bool {
    match s.item_shares.get(&caller) {
        None => false,
        Some(arr) => arr.contains(&item_id),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, get_time, ItemId, ItemMetadata, ItemType, State, User};
    use candid::Principal;

    fn setup_uploaded_file(
        state: &mut State,
        owner: Principal,
        name: &str,
        content_type: &str,
        chunks_data: Vec<Vec<u8>>,
    ) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        let total_size: u64 = chunks_data.iter().map(|c| c.len() as u64).sum();

        state.items.insert(
            item_id,
            ItemMetadata {
                id: item_id,
                name: name.to_string(),
                item_type: ItemType::File,
                parent_id: None,
                owner_principal: owner,
                created_at: current_time,
                modified_at: current_time,
                content_type: Some(content_type.to_string()),
                size: Some(total_size),
                num_chunks: Some(chunks_data.len() as u64),
            },
        );
        state.item_owners.entry(owner).or_default().push(item_id);
        for (i, chunk_data) in chunks_data.iter().enumerate() {
            state
                .file_contents
                .insert((item_id, i as u64), chunk_data.clone());
        }
        item_id
    }

    fn create_pending_file_item(state: &mut State, owner: Principal, name: &str) -> ItemId {
        let item_id = state.generate_item_id();
        let current_time = get_time();
        state.items.insert(
            item_id,
            ItemMetadata {
                id: item_id,
                name: name.to_string(),
                item_type: ItemType::File,
                parent_id: None,
                owner_principal: owner,
                created_at: current_time,
                modified_at: current_time,
                content_type: None,
                size: None,
                num_chunks: None,
            },
        );
        state.item_owners.entry(owner).or_default().push(item_id);
        state
            .file_alias_index
            .insert(format!("{}-alias", name), item_id);
        item_id
    }

    #[test]
    fn download_owned_file_correctly() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );

        let file_id = setup_uploaded_file(
            &mut state,
            owner,
            "owned.txt",
            "text/plain",
            vec![vec![1, 2, 3], vec![4, 5]],
        );

        let result_chunk0 = download_file(&state, file_id, 0, owner);
        match result_chunk0 {
            DownloadChunkResponse::FoundFileChunk(data) => {
                assert_eq!(data.contents, vec![1, 2, 3]);
                assert_eq!(data.file_type, "text/plain");
                assert_eq!(data.num_chunks, 2);
            }
            _ => panic!("Expected FoundFileChunk for chunk 0"),
        }

        let result_chunk1 = download_file(&state, file_id, 1, owner);
        match result_chunk1 {
            DownloadChunkResponse::FoundFileChunk(data) => {
                assert_eq!(data.contents, vec![4, 5]);
            }
            _ => panic!("Expected FoundFileChunk for chunk 1"),
        }
    }

    #[test]
    fn download_shared_file_correctly() {
        let mut state = State::default();
        let owner = Principal::from_slice(&[1]);
        let recipient = Principal::from_slice(&[2]);
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
            recipient,
            User {
                username: "Recipient".to_string(),
                public_key: vec![],
            },
        );

        let file_id = setup_uploaded_file(
            &mut state,
            owner,
            "shared.dat",
            "application/octet-stream",
            vec![vec![10, 20]],
        );
        state
            .item_shares
            .entry(recipient)
            .or_default()
            .push(file_id);

        let result = download_file(&state, file_id, 0, recipient);
        match result {
            DownloadChunkResponse::FoundFileChunk(data) => {
                assert_eq!(data.contents, vec![10, 20]);
                assert_eq!(data.file_type, "application/octet-stream");
            }
            _ => panic!("Expected FoundFileChunk for shared file"),
        }
    }

    #[test]
    fn download_file_not_uploaded() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        let pending_file_id = create_pending_file_item(&mut state, owner, "pending.doc");
        let result = download_file(&state, pending_file_id, 0, owner);
        assert_eq!(result, DownloadChunkResponse::NotUploadedFile);
    }

    #[test]
    fn download_file_permission_error_not_owner_or_shared() {
        let mut state = State::default();
        let owner = Principal::from_slice(&[1]);
        let other_user = Principal::from_slice(&[2]);
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
            other_user,
            User {
                username: "Other".to_string(),
                public_key: vec![],
            },
        );
        let file_id = setup_uploaded_file(
            &mut state,
            owner,
            "private.doc",
            "text/plain",
            vec![vec![1]],
        );
        let result = download_file(&state, file_id, 0, other_user);
        assert_eq!(result, DownloadChunkResponse::PermissionError);
    }

    #[test]
    fn download_folder_error() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        let folder_id = state.generate_item_id();
        state.items.insert(
            folder_id,
            ItemMetadata {
                id: folder_id,
                name: "MyFolder".to_string(),
                item_type: ItemType::Folder,
                parent_id: None,
                owner_principal: owner,
                created_at: get_time(),
                modified_at: get_time(),
                content_type: None,
                size: None,
                num_chunks: None,
            },
        );
        state.item_owners.entry(owner).or_default().push(folder_id);
        let result = download_file(&state, folder_id, 0, owner);
        assert_eq!(result, DownloadChunkResponse::NotAFile);
    }

    #[test]
    fn download_chunk_not_found() {
        let mut state = State::default();
        let owner = Principal::anonymous();
        set_user_info(
            &mut state,
            owner,
            User {
                username: "Owner".to_string(),
                public_key: vec![],
            },
        );
        let file_id = setup_uploaded_file(
            &mut state,
            owner,
            "single_chunk.txt",
            "text/plain",
            vec![vec![1, 2, 3]],
        ); // Only 1 chunk (id 0)

        // Try to download chunk 1 (which doesn't exist)
        let result = download_file(&state, file_id, 1, owner);
        assert_eq!(result, DownloadChunkResponse::ChunkNotFound);
    }
}
