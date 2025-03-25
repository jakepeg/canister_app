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
    let group_alias = state.alias_generator.next();

    // Add group alias to index
    state
        .group_alias_index
        .insert(group_alias.clone(), group_id);

    // Clone values before moving
    let group_name = input.group_name.clone();
    let file_names = input.file_names.clone();

    let mut file_ids = Vec::new();
    let mut file_aliases = Vec::new();

    // Process original file_names (consumes input.file_names)
    for file_name in input.file_names {
        let file_id = state.generate_file_id();
        let file_alias = state.alias_generator.next(); // Unique per file

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
                    alias: file_alias.clone(),
                },
            },
        );
        state.file_alias_index.insert(file_alias.clone(), file_id);
        state.file_owners.entry(caller).or_default().push(file_id);
        file_ids.push(file_id);
        file_aliases.push(file_alias);
    }

    // Create request group with original group_name
    let request_group = RequestGroup {
        group_id,
        name: input.group_name,
        files: file_ids.clone(),
        requester: caller,
        created_at: get_time(),
    };

    state.request_groups.insert(group_id, request_group);

    // Add this line to store file IDs in group_files
    state.group_files.insert(group_id, file_ids.clone());

    // Save template using cloned values
    if input.save_as_template {
        let _ = crate::api::template::save_template(state, caller, group_name, file_names);
    }

    MultiRequestResponse {
        group_id,
        group_alias,
    }
}
