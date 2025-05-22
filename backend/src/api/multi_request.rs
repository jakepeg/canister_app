// ic-docutrack/backend/src/api/multi_request.rs
use crate::{
    get_time,
    ItemMetadata,
    ItemType,
    MultiRequestInputLegacy,    // Changed from MultiRequestInput
    MultiRequestResponseLegacy, // Changed from MultiRequestResponse
    RequestGroup,
    State,
};
use candid::Principal;

pub fn multi_request(
    caller: Principal,
    input: MultiRequestInputLegacy, // Changed type
    state: &mut State,
) -> MultiRequestResponseLegacy {
    // Changed return type
    let current_time = get_time();

    let group_folder_id = state.generate_item_id();
    let group_folder_metadata = ItemMetadata {
        id: group_folder_id,
        name: input.group_name.clone(),
        item_type: ItemType::Folder,
        parent_id: None,
        owner_principal: caller,
        created_at: current_time,
        modified_at: current_time,
        content_type: None,
        size: None,
        num_chunks: None,
    };
    state.items.insert(group_folder_id, group_folder_metadata);
    state
        .item_owners
        .entry(caller)
        .or_default()
        .push(group_folder_id);

    let group_alias = state.alias_generator.next();
    state
        .group_alias_index
        .insert(group_alias.clone(), group_folder_id);

    let mut requested_item_ids_in_group = Vec::new();
    let file_names_for_template = input.file_names.clone();

    for file_name_str in input.file_names {
        let item_id = state.generate_item_id();
        let file_alias = state.alias_generator.next();

        let file_item_metadata = ItemMetadata {
            id: item_id,
            name: file_name_str,
            item_type: ItemType::File,
            parent_id: Some(group_folder_id),
            owner_principal: caller,
            created_at: current_time,
            modified_at: current_time,
            content_type: None,
            size: None,
            num_chunks: None,
        };
        state.items.insert(item_id, file_item_metadata);
        state.file_alias_index.insert(file_alias.clone(), item_id);
        state.item_owners.entry(caller).or_default().push(item_id);
        requested_item_ids_in_group.push(item_id);
    }

    let request_group_entry = RequestGroup {
        group_id: group_folder_id,
        name: input.group_name.clone(),
        files: requested_item_ids_in_group.clone(),
        requester: caller,
        created_at: current_time,
    };
    state
        .request_groups
        .insert(group_folder_id, request_group_entry);

    state
        .group_files
        .insert(group_folder_id, requested_item_ids_in_group);

    if input.save_as_template {
        let _ = crate::api::template::save_template(
            state,
            caller,
            input.group_name, // This moves input.group_name
            file_names_for_template,
        );
    }

    MultiRequestResponseLegacy {
        // Changed struct name
        group_id: group_folder_id,
        group_alias,
    }
}
