// ic-docutrack/backend/src/api/get_request_groups.rs
use crate::{
    PublicItemMetadata, // Use new PublicItemMetadata
    PublicRequestGroup,
    State,
};
use candid::Principal;

pub fn get_request_groups(state: &State, caller: Principal) -> Vec<PublicRequestGroup> {
    state
        .request_groups // This map's key is the group_folder_id
        .iter()
        .filter(|(_group_folder_id, group_info)| group_info.requester == caller)
        .map(|(group_folder_id, group_info_entry)| {
            // group_info_entry is RequestGroup
            // The files in group_info_entry.files are ItemIds of the requested files
            let files_metadata: Vec<PublicItemMetadata> = group_info_entry
                .files
                .iter()
                .filter_map(|&item_id_in_group| {
                    state.items.get(&item_id_in_group).map(|item_meta| {
                        // Convert ItemMetadata of the file to PublicItemMetadata
                        PublicItemMetadata {
                            id: item_meta.id,
                            name: item_meta.name.clone(),
                            item_type: item_meta.item_type.clone(),
                            parent_id: item_meta.parent_id, // This will be Some(group_folder_id)
                            modified_at: item_meta.modified_at,
                            size: item_meta.size,
                            // owner_principal: item_meta.owner_principal, // If needed
                        }
                    })
                })
                .collect();

            PublicRequestGroup {
                group_id: *group_folder_id, // This is the ID of the folder representing the group
                name: group_info_entry.name.clone(),
                files: files_metadata,
                created_at: group_info_entry.created_at,
            }
        })
        .collect()
}
