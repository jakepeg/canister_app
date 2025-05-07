// api/get_request_groups.rs
use crate::{PublicFileMetadata, PublicRequestGroup, State};
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

                        // Find group name for this file
                        let group_name = state
                            .request_groups
                            .values()
                            .find(|group| group.files.contains(file_id))
                            .map(|group| group.name.clone())
                            .unwrap_or_default();

                        // Find group alias for this file
                        let group_alias = state
                            .request_groups
                            .values()
                            .find(|group| group.files.contains(file_id))
                            .and_then(|group| {
                                state
                                    .group_alias_index
                                    .iter()
                                    .find(|(_a, id)| **id == group.group_id)
                                    .map(|(alias, _)| alias.clone())
                            });

                        PublicFileMetadata {
                            file_id: *file_id,
                            file_name: file_data.metadata.file_name.clone(),
                            group_name,
                            group_alias,
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
