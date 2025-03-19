// api/get_request_groups.rs
use crate::{PublicFileMetadata, PublicRequestGroup, RequestGroup, State};
use candid::Principal;

pub fn get_request_groups(state: &State, caller: Principal) -> Vec<PublicRequestGroup> {
    state
        .request_groups
        .iter()
        .filter(|(_, group)| group.requester == caller)
        .map(|(_, group)| {
            PublicRequestGroup {
                group_id: group.group_id,
                name: group.name.clone(),
                files: group
                    .files
                    .iter()
                    .map(|file_id| {
                        // Reuse the existing get_requests function logic to format file metadata
                        let file_data = state.file_data.get(file_id).expect("File must exist");
                        PublicFileMetadata {
                            file_id: *file_id,
                            file_name: file_data.metadata.file_name.clone(),
                            shared_with: super::get_requests::get_allowed_users(state, *file_id),
                            file_status: super::get_requests::get_file_status(state, *file_id),
                        }
                    })
                    .collect(),
                created_at: group.created_at,
            }
        })
        .collect()
}
