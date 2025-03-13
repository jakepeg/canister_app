use crate::{FileInfo, GetAliasInfoError, GroupInfo, PublicUser, State};

// pub struct GroupInfo {
//     pub group_id: u64,
//     pub group_name: String,
//     pub files: Vec<FileInfo>,
//     pub requester: PublicUser,
// }

// pub struct FileInfo {
//     pub file_id: u64,
//     pub file_name: String,
//     pub alias: String,
// }

pub fn get_group_by_alias(state: &State, alias: String) -> Result<GroupInfo, GetAliasInfoError> {
    // First, get the file ID from the alias
    let file_id = state
        .file_alias_index
        .get(&alias)
        .ok_or(GetAliasInfoError::NotFound)?;

    // Find which group contains this file
    let group = state
        .request_groups
        .values()
        .find(|group| group.files.contains(file_id))
        .ok_or(GetAliasInfoError::NotFound)?;

    // Get requester info
    let requester = state
        .users
        .get(&group.requester)
        .ok_or(GetAliasInfoError::NotFound)?;

    // Build the response
    let files = group
        .files
        .iter()
        .map(|&file_id| {
            let file = state.file_data.get(&file_id).unwrap();
            let alias = match &file.content {
                crate::FileContent::Pending { alias } => alias.clone(),
                _ => String::new(), // Already uploaded
            };

            FileInfo {
                file_id,
                file_name: file.metadata.file_name.clone(),
                alias,
            }
        })
        .collect();

    Ok(GroupInfo {
        group_id: group.group_id,
        group_name: group.name.clone(),
        files,
        requester: PublicUser {
            username: requester.username.clone(),
            public_key: requester.public_key.clone(),
            ic_principal: group.requester,
        },
    })
}
