// ic-docutrack/backend/src/api/create_folder.rs
use crate::{get_time, ItemId, ItemMetadata, ItemType, PublicItemMetadata, State};
use candid::Principal;

pub fn create_folder(
    state: &mut State,
    caller: Principal,
    name: String,
    parent_id: Option<ItemId>,
) -> Result<PublicItemMetadata, String> {
    // Validate parent_id if provided (ensure it's a folder and caller has permission to write into it)
    if let Some(p_id) = parent_id {
        match state.items.get(&p_id) {
            Some(parent_item) => {
                if parent_item.item_type != ItemType::Folder {
                    return Err("Parent item is not a folder.".to_string());
                }
                // Check if caller owns the parent folder or has write permissions (if such a concept exists)
                // For now, only owners can create items inside their folders.
                if parent_item.owner_principal != caller {
                    return Err(
                        "Permission denied to create folder in the specified parent folder."
                            .to_string(),
                    );
                }
            }
            None => return Err("Parent folder not found.".to_string()),
        }
    } else {
        // If parent_id is None, it's a root folder. No specific parent permission check needed beyond general auth.
    }

    let item_id = state.generate_item_id();
    let current_time = get_time();

    let folder_metadata = ItemMetadata {
        id: item_id,
        name: name.clone(),
        item_type: ItemType::Folder,
        parent_id,
        owner_principal: caller,
        created_at: current_time,
        modified_at: current_time,
        content_type: None,
        size: None,
        num_chunks: None,
    };

    state.items.insert(item_id, folder_metadata.clone());
    state.item_owners.entry(caller).or_default().push(item_id);

    Ok(PublicItemMetadata {
        id: folder_metadata.id,
        name: folder_metadata.name,
        item_type: folder_metadata.item_type,
        parent_id: folder_metadata.parent_id,
        modified_at: folder_metadata.modified_at,
        size: folder_metadata.size,
    })
}
