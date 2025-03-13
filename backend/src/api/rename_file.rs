use crate::{FileSharingResponse, State};
use candid::Principal;

pub fn rename_file(
    state: &mut State,
    caller: Principal,
    file_id: u64,
    new_name: String,
) -> FileSharingResponse {
    // Check if the file exists and the user owns it
    match state.file_owners.get(&caller) {
        Some(files) => {
            if !files.contains(&file_id) {
                return FileSharingResponse::PermissionError;
            }

            // File exists and user has permission, update the name
            if let Some(file) = state.file_data.get_mut(&file_id) {
                file.metadata.file_name = new_name;
                FileSharingResponse::Ok
            } else {
                // This shouldn't happen if file_owners is consistent
                FileSharingResponse::PermissionError
            }
        }
        None => FileSharingResponse::PermissionError,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{request_file, set_user_info},
        User,
    };
    use candid::Principal;

    #[test]
    fn rename_file_test() {
        let mut state = State::default();
        set_user_info(
            &mut state,
            Principal::anonymous(),
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        // Request a file
        request_file(Principal::anonymous(), "original_name", &mut state);

        // Rename the file
        let result = rename_file(
            &mut state,
            Principal::anonymous(),
            0,
            "new_name".to_string(),
        );

        // Verify result and new file name
        assert_eq!(result, FileSharingResponse::Ok);
        assert_eq!(
            state.file_data.get(&0).unwrap().metadata.file_name,
            "new_name"
        );
    }

    #[test]
    fn rename_file_permission_error() {
        let mut state = State::default();
        set_user_info(
            &mut state,
            Principal::anonymous(),
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        set_user_info(
            &mut state,
            Principal::from_slice(&[0, 1, 2]),
            User {
                username: "Jane".to_string(),
                public_key: vec![3, 4, 5],
            },
        );

        // Request a file as anonymous
        request_file(Principal::anonymous(), "original_name", &mut state);

        // Try to rename as another user
        let result = rename_file(
            &mut state,
            Principal::from_slice(&[0, 1, 2]),
            0,
            "new_name".to_string(),
        );

        // Verify permission error
        assert_eq!(result, FileSharingResponse::PermissionError);
        assert_eq!(
            state.file_data.get(&0).unwrap().metadata.file_name,
            "original_name"
        );
    }
}
