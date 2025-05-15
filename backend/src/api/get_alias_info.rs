// ic-docutrack/backend/src/api/get_alias_info.rs
use crate::{
    AliasInfo,
    GetAliasInfoError,
    // ItemId,
    // ItemMetadata,
    ItemType,
    PublicUser,
    State,
    // FileMetadata, // Old, remove
};

pub fn get_alias_info(state: &State, alias: String) -> Result<AliasInfo, GetAliasInfoError> {
    // 1. Find the ItemId associated with the given alias
    let item_id = state
        .file_alias_index
        .get(&alias)
        .copied() // Get a copy of the ItemId (u64 implements Copy)
        .ok_or(GetAliasInfoError::NotFound)?;

    // 2. Get the ItemMetadata for this ItemId
    let item_metadata = state
        .items
        .get(&item_id)
        .ok_or(GetAliasInfoError::NotFound)?; // Should not happen if alias_index is consistent

    // 3. Ensure the item is a File (aliases are for file uploads, not folder creation via alias)
    if item_metadata.item_type != ItemType::File {
        return Err(GetAliasInfoError::NotFound); // Or a more specific error like "AliasNotForFile"
    }

    // 4. Get the requester's (owner's) public user information
    let owner_info = state
        .users
        .get(&item_metadata.owner_principal)
        // .unwrap() // Avoid unwrap in production, handle potential inconsistency
        .cloned() // Clone the User struct
        .ok_or(GetAliasInfoError::NotFound)?; // Should not happen if owner_principal is always valid

    // 5. Construct and return the AliasInfo
    Ok(AliasInfo {
        file_id: item_metadata.id, // This is already item_id
        file_name: item_metadata.name.clone(),
        user: PublicUser {
            username: owner_info.username,
            public_key: owner_info.public_key,
            ic_principal: item_metadata.owner_principal,
        },
    })
}

// The helper function `get_file_metadata` is no longer needed as we directly access `state.items`.
// fn get_file_metadata(state: &State, file_id: u64) -> &FileMetadata { // Old function
//     &state.file_data.get(&file_id).unwrap().metadata
// }
