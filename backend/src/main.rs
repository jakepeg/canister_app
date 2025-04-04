use backend::api::UploadFileAtomicRequest;
// use backend::vetkd::{vetkd_encrypted_key, vetkd_public_key};
use backend::*;
// Import necessary types from the api module (which re-exports from canister_management)
// use backend::api::{GetUserCanistersResponse, RegisterCanisterResponse};
// Import the new response type
// use backend::api::CreateCanisterResponse;
use candid::Principal;
use ic_cdk::api::caller;
use ic_cdk_macros::{post_upgrade, pre_upgrade, query, update};

#[update]
fn set_user(username: String, public_key: Vec<u8>) -> SetUserResponse {
    if with_state(|s| backend::api::username_exists(s, username.clone())) {
        SetUserResponse::UsernameExists
    } else {
        let user = User {
            username,
            public_key,
        };
        with_state_mut(|s| backend::api::set_user_info(s, caller(), user));
        SetUserResponse::Ok
    }
}

#[query]
fn username_exists(username: String) -> bool {
    with_state(|s| backend::api::username_exists(s, username))
}

#[query]
fn who_am_i() -> WhoamiResponse {
    with_state(|s| match s.users.get(&ic_cdk::api::caller()) {
        None => WhoamiResponse::UnknownUser,
        Some(user) => WhoamiResponse::KnownUser(PublicUser {
            username: user.username.clone(),
            public_key: user.public_key.clone(),
            ic_principal: ic_cdk::api::caller(),
        }),
    })
}

#[query]
fn get_requests() -> Vec<PublicFileMetadata> {
    with_state(|s| backend::api::get_requests(s, caller()))
}

#[query]
fn get_shared_files() -> Vec<PublicFileMetadata> {
    with_state(|s| backend::api::get_shared_files(s, caller()))
}

#[query]
fn get_file_owner_principal(file_id: u64) -> Result<Vec<u8>, String> {
    with_state(|s| {
        s.file_data
            .get(&file_id)
            .map(|file| file.metadata.requester_principal.as_slice().to_vec())
            .ok_or_else(|| "File not found".to_string())
    })
}

#[query]
fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError> {
    with_state(|s| backend::api::get_alias_info(s, alias))
}

#[update]
fn upload_file(request: UploadFileRequest) -> Result<(), UploadFileError> {
    with_state_mut(|s| {
        backend::api::upload_file(
            request.file_id,
            request.file_content,
            request.file_type,
            // request.owner_key,
            request.num_chunks,
            s,
        )
    })
}

#[update]
fn upload_file_atomic(request: UploadFileAtomicRequest) -> u64 {
    with_state_mut(|s| backend::api::upload_file_atomic(caller(), request, s))
}

#[update]
fn upload_file_continue(request: UploadFileContinueRequest) {
    with_state_mut(|s| backend::api::upload_file_continue(request, s))
}

#[update]
fn request_file(request_name: String) -> String {
    with_state_mut(|s| backend::api::request_file(caller(), request_name, s))
}

#[update]
fn multi_request(input: MultiRequestInput) -> MultiRequestResponse {
    with_state_mut(|s| backend::api::multi_request(caller(), input, s))
}

#[query]
fn get_user_templates() -> Vec<Template> {
    with_state(|s| backend::api::get_user_templates(s, caller()))
}

#[query]
fn get_template(name: String) -> Result<Template, GetAliasInfoError> {
    with_state(|s| backend::api::get_template(s, caller(), name))
}

#[update]
fn delete_template(name: String) {
    with_state_mut(|s| {
        backend::api::delete_template(s, caller(), name).unwrap_or_else(|err| {
            ic_cdk::println!("Error deleting template: {:?}", err);
        })
    });
}

#[query]
fn get_request_groups() -> Vec<PublicRequestGroup> {
    with_state(|s| backend::api::get_request_groups(s, caller()))
}

#[query]
fn get_group_by_alias(alias: String) -> Result<GroupInfo, GetAliasInfoError> {
    with_state(|s| backend::api::get_group_by_alias(s, alias))
}

#[query]
fn download_file(file_id: u64, chunk_id: u64) -> FileDownloadResponse {
    with_state(|s| backend::api::download_file(s, file_id, chunk_id, caller()))
}

#[update]
fn share_file(
    user_id: Principal,
    file_id: u64,
    // file_key not needed as we have vetkeys now
    // file_key_encrypted_for_user: Vec<u8>,
) -> FileSharingResponse {
    with_state_mut(|s| backend::api::share_file(s, caller(), user_id, file_id))
}

#[update]
fn share_file_with_users(
    user_id: Vec<Principal>,
    file_id: u64,
    file_key_encrypted_for_user: Vec<Vec<u8>>,
) {
    with_state_mut(|s| {
        for (id, _key) in user_id.iter().zip(file_key_encrypted_for_user.iter()) {
            backend::api::share_file(s, caller(), *id, file_id);
        }
    });
}

#[update]
fn revoke_share(user_id: Principal, file_id: u64) -> FileSharingResponse {
    with_state_mut(|s| backend::api::revoke_share(s, caller(), user_id, file_id))
}

#[update]
fn delete_file(file_id: u64) -> FileSharingResponse {
    with_state_mut(|s| backend::api::delete_file(s, caller(), file_id))
}

#[update]
fn rename_file(file_id: u64, new_name: String) -> FileSharingResponse {
    with_state_mut(|s| backend::api::rename_file(s, caller(), file_id, new_name))
}

#[query]
fn get_users() -> GetUsersResponse {
    with_state(|s| backend::api::get_users(s, caller()))
}

// --- New Canister Management Endpoints ---

#[query]
fn get_user_canisters() -> GetUserCanistersResponse {
    // Directly call the implementation from the api module
    backend::api::get_user_canisters()
}

#[update]
async fn register_canister(canister_id: Principal, name: String) -> RegisterCanisterResponse {
    // Directly call the implementation from the api module
    // Note: This function in api is async, so we need await here.
    backend::api::register_canister(canister_id, name).await
}

#[update]
async fn create_new_file_canister(name: String) -> CreateCanisterResponse {
    // Directly call the implementation from the api module
    backend::api::create_new_file_canister(name).await
}

// --- End New Endpoints ---

#[pre_upgrade]
fn pre_upgrade() {
    backend::pre_upgrade();
}

#[post_upgrade]
fn post_upgrade() {
    backend::post_upgrade();
}

fn main() {}
