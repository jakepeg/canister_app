// use crate::aliases::{AliasGenerator, Randomness};
use crate::{
    get_time, File, FileContent, FileMetadata, MultiRequestInput, MultiRequestResponse,
    RequestGroup, State,
};
use candid::Principal;

pub fn multi_request(
    caller: Principal,
    input: MultiRequestInput,
    state: &mut State,
) -> MultiRequestResponse {
    let group_id = state.generate_group_id();
    let group_alias = state.alias_generator.next(); // Using state's generator

    let mut file_ids = Vec::new();

    // Create files without individual aliases
    for file_name in input.file_names {
        let file_id = state.generate_file_id();

        state.file_data.insert(
            file_id,
            File {
                metadata: FileMetadata {
                    file_name,
                    user_public_key: crate::api::user_info::get_user_key(state, caller),
                    requester_principal: caller,
                    requested_at: get_time(),
                    uploaded_at: None,
                },
                content: FileContent::Pending {
                    alias: group_alias.clone(),
                },
            },
        );

        state.file_owners.entry(caller).or_default().push(file_id);

        file_ids.push(file_id);
    }

    // Store group relationships
    state
        .group_alias_index
        .insert(group_alias.clone(), group_id);
    state.group_files.insert(group_id, file_ids.clone());

    // Create request group
    let request_group = RequestGroup {
        group_id,
        name: input.group_name,
        files: file_ids,
        requester: caller,
        created_at: get_time(),
    };

    state.request_groups.insert(group_id, request_group);

    MultiRequestResponse {
        group_id,
        group_alias,
    }
}
