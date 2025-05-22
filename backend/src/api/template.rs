use crate::{GetAliasInfoError, State, TemplateLegacy}; // Changed from Template
use candid::Principal;
use std::collections::BTreeMap;

pub fn save_template(
    state: &mut State,
    caller: Principal,
    name: String,
    file_names: Vec<String>,
) -> Result<(), GetAliasInfoError> {
    // Assuming GetAliasInfoError is still appropriate here
    let user_templates = state
        .user_templates
        .entry(caller)
        .or_insert_with(BTreeMap::new);

    user_templates.insert(name.clone(), TemplateLegacy { name, file_names }); // Changed struct name
    Ok(())
}

pub fn get_user_templates(state: &State, caller: Principal) -> Vec<TemplateLegacy> {
    // Changed return type
    state
        .user_templates
        .get(&caller)
        .map(|templates| templates.values().cloned().collect())
        .unwrap_or_default()
}

pub fn get_template(
    state: &State,
    caller: Principal,
    name: String,
) -> Result<TemplateLegacy, GetAliasInfoError> {
    // Changed return type
    state
        .user_templates
        .get(&caller)
        .and_then(|templates| templates.get(&name))
        .cloned()
        .ok_or(GetAliasInfoError::NotFound) // Assuming GetAliasInfoError::NotFound is the correct error type
}

pub fn delete_template(
    state: &mut State,
    caller: Principal,
    name: String,
) -> Result<(), GetAliasInfoError> {
    // Assuming GetAliasInfoError is still appropriate here
    state
        .user_templates
        .get_mut(&caller)
        .and_then(|templates| templates.remove(&name))
        .map(|_| ())
        .ok_or(GetAliasInfoError::NotFound)
}
