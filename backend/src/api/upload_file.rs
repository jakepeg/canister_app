use crate::{get_time, FileContent, State, UploadFileError};
// Not used as we aren't storing encrypted_keys while sharing anymore
// use std::collections::BTreeMap;

pub fn upload_file(
    file_id: u64,
    contents: Vec<u8>,
    file_type: String,
    // _owner_key: Vec<u8>,
    num_chunks: u64,
    state: &mut State,
) -> Result<(), UploadFileError> {
    // Fetch the file.
    let file = match state.file_data.get_mut(&file_id) {
        Some(file) => file,
        None => return Err(UploadFileError::NotRequested),
    };

    // Remove shared_keys as it's no longer needed
    // let shared_keys = BTreeMap::new();
    // Retrieve the alias associated with the file.
    let alias = match file.content {
        FileContent::Pending { ref alias } => {
            let alias = alias.clone();
            if num_chunks == 1 {
                file.content = FileContent::Uploaded {
                    file_type,
                    // No need for owner_key or shared_keys
                    // _owner_key,
                    // Remove shared_keys as it's no longer needed
                    // shared_keys,
                    num_chunks,
                };
            } else {
                file.content = FileContent::PartiallyUploaded {
                    file_type,
                    // No need for owner_key or shared_keys
                    // owner_key,
                    // Remove shared_keys as it's no longer needed
                    // shared_keys,
                    num_chunks,
                };
            }

            file.metadata.uploaded_at = Some(get_time());

            // Add file contents to stable store.
            let chunk_id = 0;
            state.file_contents.insert((file_id, chunk_id), contents);

            alias
        }
        FileContent::Uploaded { .. } | FileContent::PartiallyUploaded { .. } => {
            return Err(UploadFileError::AlreadyUploaded)
        }
    };

    // CHANGED: Only remove alias if it exists
    if !alias.is_empty() {
        state
            .file_alias_index
            .remove(&alias)
            .ok_or(UploadFileError::NotRequested)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{request_file, set_user_info, user_info::get_user_key},
        File, FileMetadata, User,
    };
    use candid::Principal;
    use maplit::btreemap;

    #[test]
    fn stored_file_in_state() {
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
        request_file(Principal::anonymous(), "request", &mut state);

        // The alias index is not empty.
        assert!(!state.file_alias_index.is_empty());

        // Upload the file, which we assume to have a file ID of zero.
        let file_id = 0;
        let _alias = upload_file(
            file_id,
            vec![1, 2, 3],
            "jpeg".to_string(),
            // Removed owner_key parameter as it's not needed for vetkd
            // vec![1, 2, 3],
            1,
            &mut state,
        );

        // The file is stored in the state.
        assert_eq!(
            state.file_data,
            btreemap! {
                file_id => File {
                    metadata: FileMetadata {
                        file_name: "request".to_string(),
                        user_public_key: get_user_key(&state, Principal::anonymous()),
                        requester_principal: Principal::anonymous(),
                        requested_at: get_time(),
                        uploaded_at: Some(get_time()),
                    },
                    content: FileContent::Uploaded {
                        file_type: "jpeg".to_string(),
                        // No need for owner_key or shared_keys
                        // owner_key: vec![1,2,3],
                        // Remove shared_keys as it's no longer needed
                        // shared_keys: BTreeMap::new(),
                        num_chunks: 1,
                    }
                }
            }
        );
        assert_eq!(state.file_contents.get(&(file_id, 0)), Some(vec![1, 2, 3]));

        // The alias index is empty.
        assert!(state.file_alias_index.is_empty());
    }
}
