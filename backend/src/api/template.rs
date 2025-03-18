use crate::{GetAliasInfoError, State, Template};
use candid::Principal;

pub fn save_template(
    state: &mut State,
    caller: Principal,
    name: String,
    file_names: Vec<String>,
) -> Result<(), GetAliasInfoError> {
    let user_templates = state
        .user_templates
        .entry(caller)
        .or_insert_with(BTreeMap::new);

    user_templates.insert(name, Template { name, file_names });
    Ok(())
}

pub fn get_user_templates(state: &State, caller: Principal) -> Vec<Template> {
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
) -> Result<Template, GetAliasInfoError> {
    state
        .user_templates
        .get(&caller)
        .and_then(|templates| templates.get(&name))
        .cloned()
        .ok_or(GetAliasInfoError::NotFound)
}

pub fn delete_template(
    state: &mut State,
    caller: Principal,
    name: String,
) -> Result<(), GetAliasInfoError> {
    state
        .user_templates
        .get_mut(&caller)
        .and_then(|templates| templates.remove(&name))
        .map(|_| ())
        .ok_or(GetAliasInfoError::NotFound)
}
