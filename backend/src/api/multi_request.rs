// api/multi_request.rs
use crate::{get_time, MultiRequestInput, MultiRequestResponse, RequestGroup, State};
use candid::Principal;

use super::request_file;

pub fn multi_request(
    caller: Principal,
    input: MultiRequestInput,
    state: &mut State,
) -> MultiRequestResponse {
    let group_id = state.generate_group_id();
    let mut file_aliases = Vec::with_capacity(input.file_names.len());
    let mut file_ids = Vec::with_capacity(input.file_names.len());

    // Create each file request
    for file_name in input.file_names {
        let alias = request_file(caller, file_name, state);

        // Find the file_id that was just created
        let file_id = *state
            .file_alias_index
            .get(&alias)
            .expect("File should exist after creation");

        file_aliases.push(alias);
        file_ids.push(file_id);
    }

    // Create the request group
    let request_group = RequestGroup {
        group_id,
        name: input.group_name,
        files: file_ids,
        requester: caller,
        created_at: get_time(),
    };

    // Add the group to the state
    state.request_groups.insert(group_id, request_group);

    MultiRequestResponse {
        group_id,
        file_aliases,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{api::set_user_info, User};
    use candid::Principal;

    #[test]
    fn multi_request_test() {
        let mut state = State::default();
        set_user_info(
            &mut state,
            Principal::anonymous(),
            User {
                username: "John".to_string(),
                public_key: vec![1, 2, 3],
            },
        );

        let input = MultiRequestInput {
            group_name: "Test Group".to_string(),
            file_names: vec![
                "Document 1".to_string(),
                "Document 2".to_string(),
                "Document 3".to_string(),
            ],
        };

        let response = multi_request(Principal::anonymous(), input, &mut state);

        // Verify the response
        assert_eq!(response.group_id, 0);
        assert_eq!(response.file_aliases.len(), 3);

        // Verify group was created
        let group = state.request_groups.get(&0).unwrap();
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.files.len(), 3);
        assert_eq!(group.requester, Principal::anonymous());
    }
}
