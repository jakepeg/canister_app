use crate::{get_time, ItemId, ItemType, State, UploadFileError};
// Not used as we aren't storing encrypted_keys while sharing anymore
// use std::collections::BTreeMap;

pub fn upload_file(
    item_id: ItemId, // Changed from u64 to ItemId
    contents: Vec<u8>,
    file_type: String,
    // _owner_key: Vec<u8>,
    num_chunks: u64,
    state: &mut State,
) -> Result<(), UploadFileError> {
    // Fetch the item metadata.
    let item = match state.items.get_mut(&item_id) {
        Some(item_meta) => item_meta,
        None => return Err(UploadFileError::NotRequested),
    };

    // Ensure it's a file we are trying to upload to
    if item.item_type != ItemType::File {
        // This error isn't in UploadFileError yet, might need a new variant or a generic error
        return Err(UploadFileError::NotRequested); // Or a more specific error
    }

    // Check if already uploaded or partially uploaded (based on content_type being Some)
    if item.content_type.is_some() {
        // If it's already fully uploaded (all chunks present), error.
        // If it's partially uploaded and this is chunk 0 again, it could be an error or a retry.
        // For simplicity, if content_type is set, we assume it's at least partially started.
        // A more robust check would see if chunk 0 for item_id exists in file_contents.
        if state.file_contents.contains_key(&(item_id, 0)) {
            return Err(UploadFileError::AlreadyUploaded); // Or "ChunkAlreadyUploaded"
        }
    }

    // Update the item metadata with file-specific details
    item.content_type = Some(file_type);
    item.num_chunks = Some(num_chunks);
    item.size = Some(contents.len() as u64); // Initial size (first chunk), will be updated if multi-chunk
    item.modified_at = get_time();
    // item.uploaded_at (old field) is now covered by modified_at for files.

    // Add file contents (first chunk) to stable store.
    let chunk_id = 0_u64; // This function handles the first chunk for an alias upload
    state.file_contents.insert((item_id, chunk_id), contents);

    // Remove the alias from the index since the upload has started.
    // We need to find the alias associated with this item_id.
    let alias_to_remove: Option<String> = state
        .file_alias_index
        .iter()
        .find(|(_, &id)| id == item_id)
        .map(|(alias, _)| alias.clone());

    if let Some(alias) = alias_to_remove {
        state.file_alias_index.remove(&alias);
    }
    // If no alias was found but we proceeded, it's an inconsistent state,
    // but the primary check was item_id existence.

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{request_file, set_user_info},
        ItemType,
        // File, FileMetadata, // Old types
        User,
    };
    use candid::Principal;

    #[test]
    fn stored_file_in_state() {
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

        // Request a file (this creates an ItemMetadata entry with no content_type/size/num_chunks yet)
        // Assuming request_file is updated to take parent_id (None for root here)
        let alias = request_file(caller, "test_request.jpg".to_string(), None, &mut state);
        let item_id = *state.file_alias_index.get(&alias).unwrap();

        // Alias index should now have the item
        assert!(!state.file_alias_index.is_empty());
        assert_eq!(state.file_alias_index.get(&alias), Some(&item_id));

        // Upload the file content to this item_id
        let upload_result = upload_file(
            item_id,
            vec![1, 2, 3, 4, 5],      // content
            "image/jpeg".to_string(), // file_type
            1,                        // num_chunks
            &mut state,
        );
        assert!(upload_result.is_ok());

        // Verify the item metadata is updated
        let stored_item = state
            .items
            .get(&item_id)
            .expect("Item not found after upload");
        assert_eq!(stored_item.id, item_id);
        assert_eq!(stored_item.name, "test_request.jpg");
        assert_eq!(stored_item.item_type, ItemType::File); // request_file should set this
        assert_eq!(stored_item.owner_principal, caller);
        assert_eq!(stored_item.content_type, Some("image/jpeg".to_string()));
        assert_eq!(stored_item.size, Some(5)); // Size of the uploaded content
        assert_eq!(stored_item.num_chunks, Some(1));
        assert!(stored_item.modified_at >= stored_item.created_at);

        // Verify file contents
        assert_eq!(
            state.file_contents.get(&(item_id, 0)),
            Some(vec![1, 2, 3, 4, 5])
        );

        // The alias index should be empty after successful upload via alias.
        assert!(state.file_alias_index.get(&alias).is_none());
    }
}
