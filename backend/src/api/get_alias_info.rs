// ic-docutrack/backend/src/api/get_alias_info.rs
use crate::{
    AliasInfoForUpload, // Changed from AliasInfo
    GetAliasInfoError,
    ItemType,
    PublicUser,
    State,
};

pub fn get_alias_info(
    state: &State,
    alias: String,
) -> Result<AliasInfoForUpload, GetAliasInfoError> {
    let item_id = state
        .file_alias_index
        .get(&alias)
        .copied()
        .ok_or(GetAliasInfoError::NotFound)?;

    let item_metadata = state
        .items
        .get(&item_id)
        .ok_or(GetAliasInfoError::NotFound)?;

    if item_metadata.item_type != ItemType::File {
        return Err(GetAliasInfoError::NotFound);
    }

    let owner_info = state
        .users
        .get(&item_metadata.owner_principal)
        .cloned()
        .ok_or(GetAliasInfoError::NotFound)?;

    Ok(AliasInfoForUpload {
        // Changed struct name
        item_id: item_metadata.id, // field name was file_id, changed to item_id for consistency with struct def
        file_name: item_metadata.name.clone(),
        user: PublicUser {
            username: owner_info.username,
            public_key: owner_info.public_key,
            ic_principal: item_metadata.owner_principal,
        },
    })
}
