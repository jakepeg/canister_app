// ic-docutrack/backend/src/api/list_folder_contents.rs
use crate::{ItemId, ItemType, PublicItemMetadata, State};
use candid::Principal;

pub fn list_folder_contents(
    state: &State,
    caller: Principal,
    folder_id: Option<ItemId>,
) -> Result<Vec<PublicItemMetadata>, String> {
    // If folder_id is Some, check if the caller has access to this folder
    if let Some(f_id) = folder_id {
        match state.items.get(&f_id) {
            Some(folder_item) => {
                if folder_item.item_type != ItemType::Folder {
                    return Err("Specified ID is not a folder.".to_string());
                }
                let is_owner = folder_item.owner_principal == caller;
                let is_shared_with_caller = state
                    .item_shares
                    .get(&caller)
                    .map_or(false, |shared_ids| shared_ids.contains(&f_id));

                if !is_owner && !is_shared_with_caller {
                    return Err("Permission denied to list folder contents.".to_string());
                }
            }
            None => return Err("Folder not found.".to_string()),
        }
    }
    // If folder_id is None, we are listing the root for the caller.
    // This means items where parent_id is None AND owned by caller OR shared with caller.

    let mut contents = Vec::new();
    for item_meta in state.items.values() {
        // Check parent_id condition
        let parent_match = item_meta.parent_id == folder_id;

        if parent_match {
            // Check ownership or share status for the item itself to be listed
            let is_owner = item_meta.owner_principal == caller;
            let is_shared_with_caller = state
                .item_shares
                .get(&caller)
                .map_or(false, |shared_ids| shared_ids.contains(&item_meta.id));

            if is_owner || is_shared_with_caller {
                contents.push(PublicItemMetadata {
                    id: item_meta.id,
                    name: item_meta.name.clone(),
                    item_type: item_meta.item_type.clone(),
                    parent_id: item_meta.parent_id,
                    modified_at: item_meta.modified_at,
                    size: item_meta.size,
                });
            }
        }
    }
    Ok(contents)
}
