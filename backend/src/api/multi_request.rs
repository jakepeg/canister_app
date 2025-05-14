// ic-docutrack/backend/src/api/multi_request.rs
use crate::{
    get_time,
    ItemMetadata,
    ItemType, // New item types
    MultiRequestInput,
    MultiRequestResponse,
    RequestGroup,
    State,
};
use candid::Principal;

// get_user_key is not used as owner_principal is the caller.
// user_public_key is not in ItemMetadata.

pub fn multi_request(
    caller: Principal,
    input: MultiRequestInput,
    state: &mut State,
) -> MultiRequestResponse {
    let current_time = get_time();

    // 1. Create the "group" as a Folder Item if it doesn't represent an existing folder.
    // For now, multi_request implies creating a new conceptual group.
    // This group could itself be a Folder Item.
    // The `parent_id` for this group folder would be `None` if multi-requests always create root groups,
    // or it could be passed in `MultiRequestInput` if groups can be nested.
    // Let's assume multi-request groups are root folders for now.
    let group_folder_id = state.generate_item_id();
    let group_folder_metadata = ItemMetadata {
        id: group_folder_id,
        name: input.group_name.clone(), // The group_name becomes the folder name
        item_type: ItemType::Folder,
        parent_id: None, // Assuming root group/folder for now
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

    // The alias for the group (folder) itself
    let group_alias = state.alias_generator.next();
    state
        .group_alias_index
        .insert(group_alias.clone(), group_folder_id); // Map alias to the folder's ID

    // 2. Create placeholder ItemMetadata for each file within this group/folder
    let mut requested_item_ids_in_group = Vec::new();
    // let mut file_aliases_for_response = Vec::new(); // Not directly part of MultiRequestResponse anymore

    // Clone file_names for template saving, as input.file_names is consumed below
    let file_names_for_template = input.file_names.clone();

    for file_name_str in input.file_names {
        // Consumes input.file_names
        let item_id = state.generate_item_id();
        let file_alias = state.alias_generator.next(); // Unique alias for each file request

        let file_item_metadata = ItemMetadata {
            id: item_id,
            name: file_name_str,
            item_type: ItemType::File,
            parent_id: Some(group_folder_id), // Files are parented to the group folder
            owner_principal: caller,
            created_at: current_time,
            modified_at: current_time,
            content_type: None,
            size: None,
            num_chunks: None,
        };
        state.items.insert(item_id, file_item_metadata);
        state.file_alias_index.insert(file_alias.clone(), item_id); // Each file request gets an alias
        state.item_owners.entry(caller).or_default().push(item_id);

        requested_item_ids_in_group.push(item_id);
        // file_aliases_for_response.push(file_alias); // Not needed for MultiRequestResponse
    }

    // 3. Create the RequestGroup entry (legacy concept, might be refactored later)
    // This RequestGroup now refers to the folder's ID and the ItemIds of files within it.
    let request_group_entry = RequestGroup {
        group_id: group_folder_id, // Use the folder's ID as the group_id
        name: input.group_name.clone(),
        files: requested_item_ids_in_group.clone(), // Store ItemIds of the files
        requester: caller,
        created_at: current_time,
    };
    state
        .request_groups
        .insert(group_folder_id, request_group_entry);

    // state.group_files maps the group_folder_id to the list of file ItemIds
    state
        .group_files
        .insert(group_folder_id, requested_item_ids_in_group);

    // 4. Save template if requested
    if input.save_as_template {
        // Use the original group_name and cloned file_names_for_template
        let _ = crate::api::template::save_template(
            state,
            caller,
            input.group_name,
            file_names_for_template,
        );
    }

    MultiRequestResponse {
        group_id: group_folder_id, // Return the ID of the created group/folder
        group_alias,               // Return the alias of the created group/folder
    }
}
