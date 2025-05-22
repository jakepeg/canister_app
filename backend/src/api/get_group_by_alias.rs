// ic-docutrack/backend/src/api/get_group_by_alias.rs
use crate::{
    FileInfoForUpload, // Changed from FileInfo
    GetAliasInfoError,
    GroupInfoForUploadResponse, // Changed from GroupInfo
    ItemType,
    PublicUser,
    State,
};

pub fn get_group_by_alias(
    state: &State,
    alias: String,
) -> Result<GroupInfoForUploadResponse, GetAliasInfoError> {
    let group_folder_id = state
        .group_alias_index
        .get(&alias)
        .copied()
        .ok_or(GetAliasInfoError::NotFound)?;

    let group_folder_metadata = state
        .items
        .get(&group_folder_id)
        .ok_or(GetAliasInfoError::NotFound)?;

    if group_folder_metadata.item_type != ItemType::Folder {
        return Err(GetAliasInfoError::NotFound);
    }

    let requester_info = state
        .users
        .get(&group_folder_metadata.owner_principal)
        .cloned()
        .ok_or(GetAliasInfoError::NotFound)?;

    let child_files_info: Vec<FileInfoForUpload> = state // Changed struct name
        .items
        .values()
        .filter(|item| item.parent_id == Some(group_folder_id) && item.item_type == ItemType::File)
        .filter_map(|file_item_meta| {
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
                .unwrap_or_else(|| "".to_string());

            Some(FileInfoForUpload {
                // Changed struct name
                item_id: file_item_meta.id, // field name was file_id
                file_name: file_item_meta.name.clone(),
                alias: individual_file_alias,
            })
        })
        .collect();

    Ok(GroupInfoForUploadResponse {
        // Changed struct name
        group_id: group_folder_id,
        group_name: group_folder_metadata.name.clone(),
        files: child_files_info,
        requester: PublicUser {
            username: requester_info.username,
            public_key: requester_info.public_key,
            ic_principal: group_folder_metadata.owner_principal,
        },
    })
}
