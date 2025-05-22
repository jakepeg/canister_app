// ic-docutrack/backend/src/api/move_item.rs
use crate::{get_time, ItemId, ItemType, State};
use candid::Principal;

pub fn move_item(
    state: &mut State,
    caller: Principal,
    item_id: ItemId,
    new_parent_id: Option<ItemId>,
) -> Result<(), String> {
    // 1. Check if item_id exists and caller owns it
    let mut item_to_move = match state.items.get(&item_id).cloned() {
        Some(item) => {
            if item.owner_principal != caller {
                return Err("Permission denied: You do not own this item.".to_string());
            }
            item
        }
        None => return Err("Item to move not found.".to_string()),
    };

    // 2. If new_parent_id is Some, check if it exists, is a folder, and caller owns it
    if let Some(p_id) = new_parent_id {
        match state.items.get(&p_id) {
            Some(parent_item) => {
                if parent_item.item_type != ItemType::Folder {
                    return Err("New parent is not a folder.".to_string());
                }
                if parent_item.owner_principal != caller {
                    return Err(
                        "Permission denied: You do not own the new parent folder.".to_string()
                    );
                }
                // 3. Prevent moving a folder into itself or one of its own descendants (cycle check)
                if item_to_move.item_type == ItemType::Folder {
                    if p_id == item_id {
                        return Err("Cannot move a folder into itself.".to_string());
                    }
                    let mut current_ancestor_id = parent_item.parent_id;
                    while let Some(ancestor_id) = current_ancestor_id {
                        if ancestor_id == item_id {
                            return Err(
                                "Cannot move a folder into one of its subfolders (cycle detected)."
                                    .to_string(),
                            );
                        }
                        current_ancestor_id =
                            state.items.get(&ancestor_id).and_then(|i| i.parent_id);
                    }
                }
            }
            None => return Err("New parent folder not found.".to_string()),
        }
    }
    // If new_parent_id is None, item is being moved to root. Caller must own the item.

    // 4. Update the item's parent_id and modified_at timestamp
    item_to_move.parent_id = new_parent_id;
    item_to_move.modified_at = get_time();
    state.items.insert(item_id, item_to_move);

    Ok(())
}
