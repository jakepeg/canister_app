use crate::{FileInfo, GetAliasInfoError, GroupInfo, PublicUser, State};

pub fn get_group_by_alias(state: &State, alias: String) -> Result<GroupInfo, GetAliasInfoError> {
    let group_id = state
        .group_alias_index
        .get(&alias)
        .ok_or(GetAliasInfoError::NotFound)?;

    let group = state
        .request_groups
        .get(group_id)
        .ok_or(GetAliasInfoError::NotFound)?;

    let files = state
        .group_files
        .get(group_id)
        .ok_or(GetAliasInfoError::NotFound)?;

    let requester = state
        .users
        .get(&group.requester)
        .ok_or(GetAliasInfoError::NotFound)?;

    Ok(GroupInfo {
        group_id: *group_id,
        group_name: group.name.clone(),
        files: files
            .iter()
            .map(|&file_id| {
                let file = state.file_data.get(&file_id).unwrap();
                FileInfo {
                    file_id,
                    file_name: file.metadata.file_name.clone(),
                    alias: String::new(), // No individual alias
                }
            })
            .collect(),
        requester: PublicUser {
            username: requester.username.clone(),
            public_key: requester.public_key.clone(),
            ic_principal: group.requester,
        },
    })
}
