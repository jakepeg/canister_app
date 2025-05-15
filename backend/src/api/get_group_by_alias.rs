// ic-docutrack/backend/src/api/get_group_by_alias.rs
use crate::{
    FileInfo,
    GetAliasInfoError,
    GroupInfo,
    // ItemId,
    // ItemMetadata,
    ItemType,
    PublicUser,
    State,
    // RequestGroup, // Not directly used here if we fetch from state.items
};
// use ic_cdk::println;

pub fn get_group_by_alias(state: &State, alias: String) -> Result<GroupInfo, GetAliasInfoError> {
    // 1. Find the group_folder_id associated with the given alias.
    //    This alias should point to an Item of ItemType::Folder.
    let group_folder_id = state
        .group_alias_index // This map stores FolderID -> Alias for groups
        .get(&alias)
        .copied()
        .ok_or(GetAliasInfoError::NotFound)?;

    // 2. Get the ItemMetadata for this group folder.
    let group_folder_metadata = state
        .items
        .get(&group_folder_id)
        .ok_or(GetAliasInfoError::NotFound)?; // Should be consistent

    // 3. Ensure the item found by alias is indeed a Folder.
    if group_folder_metadata.item_type != ItemType::Folder {
        // This alias was not for a group/folder.
        return Err(GetAliasInfoError::NotFound); // Or a more specific error.
    }

    // 4. Get the requester (owner) of this group folder.
    let requester_info = state
        .users
        .get(&group_folder_metadata.owner_principal)
        .cloned()
        .ok_or(GetAliasInfoError::NotFound)?; // Owner should exist

    // 5. Get the list of file ItemIds that are children of this group folder.
    //    These are the items whose `parent_id` matches `group_folder_id`.
    //    And they should be files intended for upload (i.e., have an alias in file_alias_index).
    let child_files_info: Vec<FileInfo> = state
        .items
        .values() // Iterate through all items
        .filter(|item| item.parent_id == Some(group_folder_id) && item.item_type == ItemType::File)
        .filter_map(|file_item_meta| {
            // For each child file, find its individual alias.
            // This assumes that files within a multi-request group also get individual aliases.
            let individual_file_alias = state
                .file_alias_index
                .iter()
                .find_map(|(f_alias, &f_id)| {
                    if f_id == file_item_meta.id {
                        Some(f_alias.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "".to_string()); // Fallback if no alias found (should not happen for requested files)

            Some(FileInfo {
                file_id: file_item_meta.id,
                file_name: file_item_meta.name.clone(),
                alias: individual_file_alias, // Each file has its own alias for upload
            })
        })
        .collect();

    // 6. Construct and return GroupInfo
    Ok(GroupInfo {
        group_id: group_folder_id, // This is the ItemId of the folder
        group_name: group_folder_metadata.name.clone(),
        files: child_files_info, // List of FileInfo for items within the folder
        requester: PublicUser {
            username: requester_info.username,
            public_key: requester_info.public_key,
            ic_principal: group_folder_metadata.owner_principal,
        },
    })
}
