// ic-docutrack/backend/src/api/download_file.rs (Consider renaming to download_item_content.rs)
use crate::{FileData, FileDownloadResponse, ItemId, ItemType, State}; // Removed FileContent, added ItemId, ItemType
use candid::Principal;

// Helper for owned item download
fn get_owned_item_data(s: &State, item_id: ItemId, chunk_id: u64) -> FileDownloadResponse {
    let item = s.items.get(&item_id).unwrap(); // Assumes item_id is valid

    if item.item_type == ItemType::Folder {
        return FileDownloadResponse::PermissionError; // Or a new "CannotDownloadFolder" variant
    }

    // Check if file has content and chunks defined (i.e., upload process started)
    let content_type = match &item.content_type {
        Some(ct) => ct.clone(),
        None => return FileDownloadResponse::NotUploadedFile, // File request exists but no upload started
    };
    let num_chunks_total = match item.num_chunks {
        Some(nc) => nc,
        None => return FileDownloadResponse::NotUploadedFile, // Should not happen if content_type is Some
    };

    // Check if all chunks are uploaded
    if s.num_chunks_uploaded(item_id) < num_chunks_total {
        return FileDownloadResponse::NotUploadedFile; // Partially uploaded
    }

    match s.file_contents.get(&(item_id, chunk_id)) {
        Some(contents) => FileDownloadResponse::FoundFile(FileData {
            contents: contents.clone(), // Clone the Vec<u8>
            file_type: content_type,
            num_chunks: num_chunks_total,
        }),
        None => FileDownloadResponse::NotFoundFile, // Specific chunk not found, though previous checks should catch this.
    }
}

// Helper for shared item download (currently same logic as owned, VetKD handles decryption differences)
fn get_shared_item_data(
    s: &State,
    item_id: ItemId,
    chunk_id: u64,
    _user: Principal,
) -> FileDownloadResponse {
    // For VetKD, the decryption key derivation involves the _user's principal on the client-side.
    // The backend serves the same encrypted content regardless of who is downloading (if they have permission).
    // So, this function can be very similar to get_owned_item_data.
    get_owned_item_data(s, item_id, chunk_id)
}

pub fn download_file(
    // Consider renaming to download_file_chunk or download_item_content
    s: &State,
    item_id: ItemId, // Changed from u64
    chunk_id: u64,
    caller: Principal,
) -> FileDownloadResponse {
    // 1. Get ItemMetadata
    let item = match s.items.get(&item_id) {
        Some(meta) => meta,
        None => return FileDownloadResponse::NotFoundFile,
    };

    // 2. Check permissions: Owner or Shared With
    let is_owner = item.owner_principal == caller;
    let is_shared = is_item_shared_with_me(s, item_id, caller);

    if !is_owner && !is_shared {
        return FileDownloadResponse::PermissionError;
    }

    // 3. If it's a folder, cannot download content
    if item.item_type == ItemType::Folder {
        return FileDownloadResponse::PermissionError; // Or specific "CannotDownloadFolder"
    }

    // 4. Check if the file is fully uploaded
    // (content_type and num_chunks should be Some, and all chunks should be present)
    if item.content_type.is_none() || item.num_chunks.is_none() {
        return FileDownloadResponse::NotUploadedFile; // Still pending or corrupt metadata
    }
    let total_chunks_expected = item.num_chunks.unwrap();
    if s.num_chunks_uploaded(item_id) < total_chunks_expected {
        return FileDownloadResponse::NotUploadedFile; // Partially uploaded
    }

    // 5. Get the specific chunk content
    match s.file_contents.get(&(item_id, chunk_id)) {
        Some(contents) => FileDownloadResponse::FoundFile(FileData {
            contents: contents.clone(), // Clone the chunk data
            file_type: item.content_type.as_ref().unwrap().clone(), // Known to be Some from check above
            num_chunks: total_chunks_expected,
        }),
        None => FileDownloadResponse::NotFoundFile, // Requested chunk_id does not exist
    }
}

// Renamed from is_file_shared_with_me
fn is_item_shared_with_me(s: &State, item_id: ItemId, caller: Principal) -> bool {
    match s.item_shares.get(&caller) {
        // Use item_shares
        None => false,
        Some(arr) => arr.contains(&item_id),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        // api::{request_file, share_file, upload_file, set_user_info}, // These need full refactoring
        api::set_user_info,
        get_time,
        // FileSharingResponse, // Added new types
        ItemId,
        ItemMetadata,
        ItemType,
        State,
        User,
    };
    use candid::Principal;

    // Helper for test setup (similar to one in delete_item.rs tests)
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

    // Helper function to create a pending file item (created via request_file typically)
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
        // Simulate alias creation for pending status
        state
            .file_alias_index
            .insert(format!("{}-alias", name), item_id);
        item_id
    }

    #[test]
    fn download_owned_file_correctly() {
        // Renamed
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

        // Download chunk 0
        let result_chunk0 = download_file(&state, file_id, 0, owner);
        match result_chunk0 {
            FileDownloadResponse::FoundFile(data) => {
                assert_eq!(data.contents, vec![1, 2, 3]);
                assert_eq!(data.file_type, "text/plain");
                assert_eq!(data.num_chunks, 2);
            }
            _ => panic!("Expected FoundFile for chunk 0"),
        }

        // Download chunk 1
        let result_chunk1 = download_file(&state, file_id, 1, owner);
        match result_chunk1 {
            FileDownloadResponse::FoundFile(data) => {
                assert_eq!(data.contents, vec![4, 5]);
            }
            _ => panic!("Expected FoundFile for chunk 1"),
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

        // Share the item (using a simplified direct share for testing)
        state
            .item_shares
            .entry(recipient)
            .or_default()
            .push(file_id);

        let result = download_file(&state, file_id, 0, recipient);
        match result {
            FileDownloadResponse::FoundFile(data) => {
                assert_eq!(data.contents, vec![10, 20]);
                assert_eq!(data.file_type, "application/octet-stream");
            }
            _ => panic!("Expected FoundFile for shared file"),
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
        assert_eq!(result, FileDownloadResponse::NotUploadedFile);
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
        assert_eq!(result, FileDownloadResponse::PermissionError);
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
        assert_eq!(result, FileDownloadResponse::PermissionError); // Or custom CannotDownloadFolder
    }
}
