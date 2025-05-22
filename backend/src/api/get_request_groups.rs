// ic-docutrack/backend/src/api/get_request_groups.rs
use crate::{
    PublicItemMetadata,
    PublicRequestGroupLegacy, // Changed from PublicRequestGroup
    State,
};
use candid::Principal;

pub fn get_request_groups(state: &State, caller: Principal) -> Vec<PublicRequestGroupLegacy> {
    // Changed return type
    state
        .request_groups
        .iter()
        .filter(|(_group_folder_id, group_info)| group_info.requester == caller)
        .map(|(group_folder_id, group_info_entry)| {
            let files_metadata: Vec<PublicItemMetadata> = group_info_entry
                .files
                .iter()
                .filter_map(|&item_id_in_group| {
                    state
                        .items
                        .get(&item_id_in_group)
                        .map(|item_meta| PublicItemMetadata {
                            id: item_meta.id,
                            name: item_meta.name.clone(),
                            item_type: item_meta.item_type.clone(),
                            parent_id: item_meta.parent_id,
                            modified_at: item_meta.modified_at,
                            size: item_meta.size,
                        })
                })
                .collect();

            PublicRequestGroupLegacy {
                // Changed struct name
                group_id: *group_folder_id,
                name: group_info_entry.name.clone(),
                files: files_metadata,
                created_at: group_info_entry.created_at,
            }
        })
        .collect()
}
