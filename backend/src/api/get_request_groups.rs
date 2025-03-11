// api/get_request_groups.rs
use crate::{PublicRequestGroup, RequestGroup, State};
use candid::Principal;

// #[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct PublicRequestGroup {
//     pub group_id: u64,
//     pub name: String,
//     pub files: Vec<PublicFileMetadata>,
//     pub created_at: u64,
// }

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        api::{multi_request, set_user_info},
        get_time, MultiRequestInput, User,
    };
    use candid::Principal;

    #[test]
    fn get_request_groups_test() {
        let mut state = State::default();
        set_user_info(
            &mut state,
            Principal::anonymous(),
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        // Create a request group
        let input = MultiRequestInput {
            group_name: "Test Group".to_string(),
            file_names: vec!["Document 1".to_string(), "Document 2".to_string()],
        };

        multi_request(Principal::anonymous(), input, &mut state);

        // Get request groups
        let groups = get_request_groups(&state, Principal::anonymous());

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Test Group");
        assert_eq!(groups[0].files.len(), 2);
        assert_eq!(groups[0].files[0].file_name, "Document 1");
        assert_eq!(groups[0].files[1].file_name, "Document 2");
    }
}
