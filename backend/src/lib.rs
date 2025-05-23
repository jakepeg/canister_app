// ic-docutrack/backend/src/lib.rs
mod aliases;
pub mod api;
mod memory;
mod upgrade;
use crate::aliases::{AliasGenerator, Randomness};
use candid::CandidType;
use candid::Principal;
use ic_stable_structures::{
    memory_manager::MemoryId,
    storable::Storable, // Import Bound from storable submodule
    StableBTreeMap,
};
use memory::{get_user_canisters_memory, Memory}; // Assuming get_user_canisters_memory will be added to memory.rs
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};
pub use upgrade::{post_upgrade, pre_upgrade};
mod declarations;
pub mod vetkd;

// --- New Canister Management Structs ---

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanisterInfo {
    pub id: Principal,
    pub name: String,
}

impl Storable for CanisterInfo {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128 + 256,
            is_fixed_size: false,
        };
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct CanisterInfoVec(Vec<CanisterInfo>);

impl Storable for CanisterInfoVec {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap_or_default()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 100 * (128 + 256) + 1024,
            is_fixed_size: false,
        };
}

const USER_CANISTERS_MEMORY_ID: MemoryId = MemoryId::new(3);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new(&get_randomness_seed()[..]));
    static USER_CANISTERS: RefCell<StableBTreeMap<Principal, CanisterInfoVec, Memory>> = RefCell::new(
        StableBTreeMap::init(get_user_canisters_memory())
    );
}

pub type ItemId = u64;
type ChunkId = u64;

pub fn with_user_canisters<R>(
    f: impl FnOnce(&StableBTreeMap<Principal, CanisterInfoVec, Memory>) -> R,
) -> R {
    USER_CANISTERS.with(|p| f(&p.borrow()))
}

pub fn with_user_canisters_mut<R>(
    f: impl FnOnce(&mut StableBTreeMap<Principal, CanisterInfoVec, Memory>) -> R,
) -> R {
    USER_CANISTERS.with(|p| f(&mut p.borrow_mut()))
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    File,
    Folder,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ItemMetadata {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>,
    pub owner_principal: Principal,
    pub created_at: u64,
    pub modified_at: u64,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub num_chunks: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicItemMetadata {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>,
    pub modified_at: u64,
    pub size: Option<u64>,
}

// --- DTOs matching main.rs and service.did ---

// Request Structs
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UploadFileAtomicDirectRequest {
    // Matches upload_file_atomic_request_new from DID
    pub name: String,
    pub content: Vec<u8>,
    pub file_type: String,
    pub num_chunks: u64,
    pub parent_id: Option<ItemId>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UploadContentToItemRequest {
    // Matches upload_file_to_item_request from DID
    pub item_id: ItemId,
    pub file_content: Vec<u8>,
    pub file_type: String,
    pub num_chunks: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadChunkContinueRequest {
    // Matches upload_chunk_continue_request from DID
    pub item_id: ItemId, // field name was file_id, changed to item_id to match DID and main.rs expectations
    pub chunk_id: u64,
    pub contents: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MultiRequestInputLegacy {
    // Matches multi_request_input from DID
    pub group_name: String,
    pub file_names: Vec<String>,
    pub save_as_template: bool,
    // parent_id: Option<ItemId>, // Optional field from DID, main.rs does not seem to use it. Keeping it simple for now.
}

// Response Structs
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FoundFileChunk {
    // Matches found_file_chunk from DID
    pub contents: Vec<u8>,
    pub file_type: String,
    pub num_chunks: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DownloadChunkResponse {
    // Matches download_file_chunk_response from DID
    #[serde(rename = "not_found_item")]
    NotFoundItem,
    #[serde(rename = "not_a_file")]
    NotAFile,
    #[serde(rename = "not_uploaded_file")]
    NotUploadedFile,
    #[serde(rename = "chunk_not_found")]
    ChunkNotFound,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "found_file_chunk")]
    FoundFileChunk(FoundFileChunk),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemOperationResponse {
    // Matches item_operation_response from DID
    Ok, // Ok: null in DID becomes Option<()> or a unit struct
    Err(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DetailedUploadResponse {
    // Matches item_operation_response_detailed from DID
    Ok(Option<()>),
    PermissionError(Option<()>),
    ItemNotFound(Option<()>),
    FolderNotEmpty(Option<()>),
    NotAFile(Option<()>),
    NotAFolder(Option<()>),
    PendingError(Option<()>),
    AlreadyUploaded(Option<()>),
    NotRequested(Option<()>),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicUser {
    // Already present, just confirming
    pub username: String,
    pub public_key: Vec<u8>,
    pub ic_principal: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AliasInfoForUpload {
    // Matches alias_info_response from DID
    pub item_id: ItemId,
    pub file_name: String,
    pub user: PublicUser,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetAliasInfoError {
    // Matches get_alias_info_error from DID
    #[serde(rename = "not_found")]
    NotFound,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum VetkdEncryptedKeyResponse {
    Ok(Vec<u8>),
    Err(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum VetkdPublicKeyResponse {
    Ok(Vec<u8>),
    Err(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MultiRequestResponseLegacy {
    // Matches multi_request_response from DID
    pub group_id: ItemId,
    pub group_alias: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicRequestGroupLegacy {
    // Matches public_request_group from DID
    pub group_id: ItemId,
    pub name: String,
    pub files: Vec<PublicItemMetadata>,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FileInfoForUpload {
    // Matches file_info_for_upload from DID
    pub item_id: ItemId,
    pub file_name: String,
    pub alias: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GroupInfoForUploadResponse {
    // Matches group_info_response from DID
    pub group_id: ItemId,
    pub group_name: String,
    pub files: Vec<FileInfoForUpload>,
    pub requester: PublicUser,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TemplateLegacy {
    // Matches template from DID
    pub name: String,
    pub file_names: Vec<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TemplateResponseLegacyError {
    // For Err variant of TemplateResponseLegacy
    #[serde(rename = "not_found")]
    NotFound,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TemplateResponseLegacy {
    // Matches template_response from DID
    Ok(TemplateLegacy),
    Err(TemplateResponseLegacyError),
}

// --- Legacy / Internal Structs (may be mapped to new DTOs or used internally) ---
// These are kept if main.rs still depends on them for mapping internal logic.

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RequestGroup {
    // Used internally by multi_request
    pub group_id: ItemId, // ItemId of the folder representing the group
    pub name: String,
    pub files: Vec<ItemId>, // ItemIds of files in this group
    pub requester: Principal,
    pub created_at: u64,
}

// MultiRequestInput and MultiRequestResponse are now suffixed with Legacy
// as they pertain to the older "group" concept.

// FileInfo and GroupInfo are now FileInfoForUpload and GroupInfoForUploadResponse for clarity

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    // Internal User struct
    pub username: String,
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum SetUserResponse {
    // For set_user endpoint
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "username_exists")]
    UsernameExists,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum WhoamiResponse {
    // For who_am_i endpoint
    #[serde(rename = "known_user")]
    KnownUser(PublicUser),
    #[serde(rename = "unknown_user")]
    UnknownUser,
}

// FileMetadata (old, largely replaced by ItemMetadata logic)
// If any API still uses it internally before converting, it can stay.
// For now, assuming it's not directly needed in the refactored state struct.

// FileStatus (old, status is now derived)
// Kept if get_requests.rs still uses it for mapping temporarily
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    #[serde(rename = "pending")]
    Pending { alias: String, requested_at: u64 },
    #[serde(rename = "partially_uploaded")]
    PartiallyUploaded,
    #[serde(rename = "uploaded")]
    Uploaded { uploaded_at: u64 },
}

// PublicFileMetadata (old, replaced by PublicItemMetadata)
// Kept if get_requests.rs still uses it for mapping temporarily
// #[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct PublicFileMetadata {
//     pub file_id: u64,
//     pub file_name: String,
//     pub group_name: String,
//     pub group_alias: Option<String>,
//     pub file_status: FileStatus,
//     pub shared_with: Vec<PublicUser>,
// }

// AliasInfo is now AliasInfoForUpload

// File and FileContent (old, replaced by ItemMetadata and derived status)
// These are removed from the State struct.

// FileData is now FoundFileChunk
// FileDownloadResponse is now DownloadChunkResponse

// Internal Error Enums (used by api modules, mapped in main.rs)
#[derive(Debug, CandidType, Serialize, Deserialize)]
pub enum UploadFileError {
    #[serde(rename = "not_requested")]
    NotRequested,
    #[serde(rename = "already_uploaded")]
    AlreadyUploaded,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum FileSharingResponse {
    #[serde(rename = "pending_error")]
    PendingError,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "ok")]
    Ok,
}

// Template is now TemplateLegacy

#[derive(Serialize, Deserialize)]
pub struct State {
    item_id_counter: u64,
    pub users: BTreeMap<Principal, User>,
    pub items: BTreeMap<ItemId, ItemMetadata>,
    pub file_alias_index: BTreeMap<String, ItemId>,
    pub item_owners: BTreeMap<Principal, Vec<ItemId>>,
    pub item_shares: BTreeMap<Principal, Vec<ItemId>>,
    #[serde(skip, default = "init_file_contents")]
    pub file_contents: StableBTreeMap<(ItemId, ChunkId), Vec<u8>, Memory>,
    #[serde(skip, default = "init_alias_generator")]
    alias_generator: AliasGenerator,
    group_count: u64, // For legacy multi_request group IDs (which are folder ItemIds now)
    // request_groups maps the group_folder_id to RequestGroup details
    pub request_groups: BTreeMap<ItemId, RequestGroup>, // Key is ItemId of the folder
    // group_alias_index maps an alias string to a group_folder_id (ItemId)
    group_alias_index: BTreeMap<String, ItemId>, // Value is ItemId of the folder
    // group_files maps a group_folder_id (ItemId) to Vec<ItemId> of files within it
    group_files: BTreeMap<ItemId, Vec<ItemId>>, // Key and Value are ItemId
    user_templates: BTreeMap<Principal, BTreeMap<String, TemplateLegacy>>,
}

impl State {
    pub(crate) fn generate_item_id(&mut self) -> ItemId {
        let item_id = self.item_id_counter;
        self.item_id_counter += 1;
        item_id
    }

    // This function is for the legacy "group_id" which is now just an ItemId for a folder.
    // It might be redundant if generate_item_id() is used for creating group folders.
    // However, multi_request.rs uses it for a conceptual group ID.
    // For consistency, let's assume group folders also get IDs from generate_item_id.
    // If group_count is truly separate, it needs careful handling.
    // Given current multi_request.rs creates a folder with generate_item_id,
    // this generate_group_id might be for the legacy RequestGroup struct's key if it's not the folder's ItemId.
    // The current multi_request.rs uses the folder_id as the key for request_groups.
    // So, generate_group_id may not be needed if group_id in RequestGroup is the folder's ItemId.
    // The provided multi_request.rs uses group_folder_id for RequestGroup.group_id.
    // Let's remove generate_group_id to avoid confusion, assuming folder IDs are sufficient.
    /*
    pub(crate) fn generate_group_id(&mut self) -> u64 { // Or ItemId
        let group_id = self.group_count;
        self.group_count += 1;
        group_id
    }
    */

    fn new(rand_seed: &[u8]) -> Self {
        Self {
            item_id_counter: 0,
            users: BTreeMap::new(),
            items: BTreeMap::new(),
            file_alias_index: BTreeMap::new(),
            item_owners: BTreeMap::new(),
            item_shares: BTreeMap::new(),
            alias_generator: AliasGenerator::new(Randomness::try_from(rand_seed).unwrap()),
            file_contents: init_file_contents(),
            group_count: 0, // Potentially unused if group IDs are just item IDs
            request_groups: BTreeMap::new(),
            group_alias_index: BTreeMap::new(),
            group_files: BTreeMap::new(),
            user_templates: BTreeMap::new(),
        }
    }

    pub(crate) fn num_chunks_uploaded(&self, item_id: ItemId) -> u64 {
        self.file_contents
            .range((Included((item_id, 0u64)), Excluded(((item_id + 1), 0u64))))
            .count() as u64
    }
}

impl Default for State {
    fn default() -> Self {
        State::new(vec![0; 32].as_slice())
    }
}

pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}

pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

pub fn generate_alias() -> String {
    with_state_mut(|s| s.alias_generator.next())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GetUsersResponse {
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "users")]
    Users(Vec<PublicUser>),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RegisterCanisterResponse {
    Ok,
    NotAuthorized,
    VerificationFailed(String),
    AlreadyRegistered,
    InternalError(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetUserCanistersResponse {
    Ok(Vec<CanisterInfo>),
    NotAuthenticated,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RenameCanisterResponse {
    Ok,
    NotAuthorized,
    CanisterNotFound,
    InternalError(String),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DeleteCanisterResponse {
    Ok,
    NotAuthorized,
    CanisterNotFound,
    DeletionFailed(String),
    InternalError(String),
}

// UploadFileRequest from old lib.rs is replaced by UploadContentToItemRequest
// UploadFileContinueRequest is updated to use item_id

#[cfg(target_arch = "wasm32")]
pub fn get_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_time() -> u64 {
    12345
}

fn get_randomness_seed() -> Vec<u8> {
    let time_seed = ic_cdk::api::time().to_be_bytes();
    let zeroes_arr: [u8; 24] = [0; 24];
    [&time_seed[..], &zeroes_arr[..]].concat()
}

fn init_alias_generator() -> AliasGenerator {
    AliasGenerator::new(Randomness::try_from(get_randomness_seed().as_slice()).unwrap())
}

pub fn ceil_division(dividend: usize, divisor: usize) -> usize {
    if dividend % divisor == 0 {
        dividend / divisor
    } else {
        dividend / divisor + 1
    }
}

fn init_file_contents() -> StableBTreeMap<(ItemId, ChunkId), Vec<u8>, Memory> {
    StableBTreeMap::init(crate::memory::get_file_contents_memory())
}
