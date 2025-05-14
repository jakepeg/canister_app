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
    // Add other relevant metadata if needed in the future, e.g., creation_timestamp
}

// Implement Storable and BoundedStorable for StableBTreeMap compatibility
impl Storable for CanisterInfo {
    fn to_bytes(&self) -> Cow<[u8]> {
        // Using ciborium for efficient encoding
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            // Explicitly qualify Bound
            // Max size for Principal text representation (~63 chars) + name + overhead
            max_size: 128 + 256, // Example: Max 256 chars for name
            is_fixed_size: false,
        };
}

// Define a wrapper type for Vec<CanisterInfo> to implement Storable
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
            // Explicitly qualify Bound
            // Estimate max size: N canisters * (size of CanisterInfo) + Vec overhead
            max_size: 100 * (128 + 256) + 1024, // Example: Max 100 canisters per user
            is_fixed_size: false,
        };
}

// --- End New Structs ---

// Memory IDs - Assuming existing IDs are 0, 1, 2 in memory.rs
const USER_CANISTERS_MEMORY_ID: MemoryId = MemoryId::new(3); // Ensure this ID is unique

thread_local! {
    /// Initialize the state randomness with the current time.
    static STATE: RefCell<State> = RefCell::new(State::new(&get_randomness_seed()[..]));

    // The memory manager is used for managing memory chunks
    // Assuming MEMORY_MANAGER is already defined similarly in memory.rs or here
    // If not, it needs to be defined like this:
    // static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    //     RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // The StableBTreeMap for user canisters
    static USER_CANISTERS: RefCell<StableBTreeMap<Principal, CanisterInfoVec, Memory>> = RefCell::new(
        StableBTreeMap::init(get_user_canisters_memory()) // Use helper from memory.rs
    );
}

type ItemId = u64;
type ChunkId = u64;

// --- Helper functions for new stable map ---

/// A helper method to read the user canisters map.
pub fn with_user_canisters<R>(
    f: impl FnOnce(&StableBTreeMap<Principal, CanisterInfoVec, Memory>) -> R,
) -> R {
    USER_CANISTERS.with(|p| f(&p.borrow()))
}

/// A helper method to mutate the user canisters map.
pub fn with_user_canisters_mut<R>(
    f: impl FnOnce(&mut StableBTreeMap<Principal, CanisterInfoVec, Memory>) -> R,
) -> R {
    USER_CANISTERS.with(|p| f(&mut p.borrow_mut()))
}

// --- End Helper Functions ---

// --- START: Hierarchical File System Changes ---

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    File,
    Folder,
}

/// Metadata for an item (file or folder) in the hierarchical system.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ItemMetadata {
    // --- Core Hierarchy Fields ---
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>, // None for root items

    // --- Common Fields ---
    pub owner_principal: Principal,
    pub created_at: u64,  // Timestamp for creation
    pub modified_at: u64, // Timestamp for last modification

    // --- File-Specific Fields (will be None for Folders) ---
    pub content_type: Option<String>, // Mime type like "image/jpeg" for files
    pub size: Option<u64>,            // File size in bytes for files
    pub num_chunks: Option<u64>,      // Total number of chunks for the file
}

/// Public representation of an item's metadata, returned to the frontend.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicItemMetadata {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>,
    pub modified_at: u64,
    pub size: Option<u64>, // For files, None for folders
                           // pub owner_principal: Principal, // Consider adding if frontend needs it for display directly
}

// The FileContent enum might be simplified or its data incorporated elsewhere
// For now, its direct usage in `State` via `File` struct is removed.
// Information like num_chunks and file_type (content_type) are now in ItemMetadata.
// The `Pending { alias }` state is handled by `file_alias_index` and items not yet having content metadata.
/*
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileContent { // This might be removed or significantly refactored
    Pending {
        alias: String,
    },
    Uploaded {
        num_chunks: u64, // Now in ItemMetadata
        file_type: String, // Now in ItemMetadata as content_type
    },
    PartiallyUploaded {
        num_chunks: u64, // Now in ItemMetadata
        file_type: String, // Now in ItemMetadata as content_type
    },
}
*/

// The `File` struct is removed. `ItemMetadata` is stored directly in the state.
/*
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    pub metadata: FileMetadata, // Old FileMetadata, to be replaced by ItemMetadata logic
    pub content: FileContent,
}
*/

// --- END: Hierarchical File System Changes ---

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RequestGroup {
    pub group_id: u64,
    pub name: String,
    pub files: Vec<u64>, // file_ids in this group
    pub requester: Principal,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MultiRequestInput {
    pub group_name: String,
    pub file_names: Vec<String>,
    pub save_as_template: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MultiRequestResponse {
    pub group_id: u64,
    pub group_alias: String, // Changed from file_aliases
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicRequestGroup {
    pub group_id: u64,
    pub name: String,
    pub files: Vec<PublicItemMetadata>,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FileInfo {
    pub file_id: ItemId,
    pub file_name: String,
    pub alias: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GroupInfo {
    pub group_id: u64,
    pub group_name: String,
    pub files: Vec<FileInfo>,
    pub requester: PublicUser,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum SetUserResponse {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "username_exists")]
    UsernameExists,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum WhoamiResponse {
    #[serde(rename = "known_user")]
    KnownUser(PublicUser),
    #[serde(rename = "unknown_user")]
    UnknownUser,
}

/// File metadata.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FileMetadata {
    pub file_name: String,
    pub user_public_key: Vec<u8>,
    pub requester_principal: Principal,
    pub requested_at: u64,
    pub uploaded_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    #[serde(rename = "pending")]
    Pending { alias: String, requested_at: u64 },
    #[serde(rename = "partially_uploaded")]
    PartiallyUploaded,
    #[serde(rename = "uploaded")]
    Uploaded {
        uploaded_at: u64,
        // No document_key needed here as we moved to vertkeys
        // document_key: Vec<u8>,
    },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicFileMetadata {
    pub file_id: u64,
    pub file_name: String,
    pub group_name: String,
    pub group_alias: Option<String>,
    pub file_status: FileStatus,
    pub shared_with: Vec<PublicUser>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetAliasInfoError {
    #[serde(rename = "not_found")]
    NotFound,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AliasInfo {
    pub file_id: u64,
    pub file_name: String,
    pub user: PublicUser,
}

// A file is composed of its metadata and its content, which is a blob.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    pub metadata: FileMetadata,
    pub content: FileContent,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileContent {
    Pending {
        alias: String,
    },
    Uploaded {
        num_chunks: u64,
        file_type: String,
        // owner_key: Vec<u8>, // VetKD public key
        // No need for shared_keys map as we are moving to vetkeys
        // shared_keys: BTreeMap<Principal, Vec<u8>>,
    },
    PartiallyUploaded {
        num_chunks: u64,
        file_type: String,
        // owner_key: Vec<u8>, // VetKD public key
        // No need for shared_keys map as we are moving to vetkeys
        // shared_keys: BTreeMap<Principal, Vec<u8>>,
    },
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub struct FileData {
    contents: Vec<u8>,
    file_type: String,
    // Remove owner_key field as it's not needed with VetKD
    // owner_key: Vec<u8>,
    num_chunks: u64,
}

#[derive(CandidType, Serialize, Deserialize, PartialEq, Debug)]
pub enum FileDownloadResponse {
    #[serde(rename = "not_found_file")]
    NotFoundFile,
    #[serde(rename = "not_uploaded_file")]
    NotUploadedFile,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "found_file")]
    FoundFile(FileData),
}

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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub file_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    // Keeps track of how many files have been requested so far
    // and is used to assign IDs to newly requested files.
    // file_count: u64,
    item_id_counter: u64, // Renamed from file_count

    /// Keeps track of usernames vs. their principals.
    pub users: BTreeMap<Principal, User>,

    /// Mapping between items IDs and item information.
    pub items: BTreeMap<ItemId, ItemMetadata>, // New map for all items (was pub file_data: BTreeMap<u64, File> before)

    /// Mapping between file aliases (randomly generated links) and item ID.
    /// This is primarily for alias-based file uploads.
    pub file_alias_index: BTreeMap<String, ItemId>,

    /// Mapping between a user's principal and the list of items (files/folders) owned by the user.
    pub item_owners: BTreeMap<Principal, Vec<ItemId>>, // Renamed from file_owners

    /// Mapping between a user's principal and the list of items (files/folders) shared with them.
    pub item_shares: BTreeMap<Principal, Vec<ItemId>>, // Renamed from file_shares

    /// The contents of the file (stored in stable memory).
    #[serde(skip, default = "init_file_contents")]
    pub file_contents: StableBTreeMap<(ItemId, ChunkId), Vec<u8>, Memory>, // Key is (ItemId, ChunkId)

    // Generates aliases for file requests.
    #[serde(skip, default = "init_alias_generator")]
    alias_generator: AliasGenerator,

    /// Counter for group IDs
    group_count: u64,

    /// Mapping between group IDs and request groups
    pub request_groups: BTreeMap<u64, RequestGroup>,

    /// Mapping between group aliases and group IDs
    group_alias_index: BTreeMap<String, u64>,
    /// Mapping between group IDs and their file IDs
    group_files: BTreeMap<u64, Vec<ItemId>>,

    user_templates: BTreeMap<Principal, BTreeMap<String, Template>>,
    // Note: user_canisters map is now managed separately via USER_CANISTERS thread_local
}

impl State {
    // Note: State::new might not need to initialize anything related to user_canisters
    // if it's purely managed by the thread_local static.
    pub(crate) fn generate_item_id(&mut self) -> ItemId {
        // The file ID is an auto-incrementing integer.

        let file_id = self.item_id_counter;
        self.item_id_counter += 1;
        file_id
    }

    pub(crate) fn generate_group_id(&mut self) -> u64 {
        let group_id = self.group_count;
        self.group_count += 1;
        group_id
    }

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
            group_count: 0,
            request_groups: BTreeMap::new(),
            group_alias_index: BTreeMap::new(),
            group_files: BTreeMap::new(),
            user_templates: BTreeMap::new(),
        }
    }

    /// Returns the number of uploaded chunks for the given file id
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

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}

/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

/// Returns an unused file alias.
pub fn generate_alias() -> String {
    with_state_mut(|s| s.alias_generator.next())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicUser {
    pub username: String,
    pub public_key: Vec<u8>,
    pub ic_principal: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum GetUsersResponse {
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "users")]
    Users(Vec<PublicUser>),
}

// --- New Canister Management Response Types (Moved from canister_management.rs) ---
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RegisterCanisterResponse {
    Ok,
    NotAuthorized,              // If caller doesn't control the canister_id
    VerificationFailed(String), // If canister_status call fails
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
// --- End New Types ---

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileRequest {
    pub file_id: ItemId,
    pub file_content: Vec<u8>,
    pub file_type: String,
    // Not needed for VetKD
    // pub owner_key: Vec<u8>,
    pub num_chunks: u64,
    // pub parent_id: Option<ItemId>, // Required for new plan
    // pub name: Option<String>, // Required if uploading a new file not tied to an alias
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileContinueRequest {
    pub file_id: ItemId,
    pub chunk_id: u64,
    pub contents: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
pub fn get_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_time() -> u64 {
    // This is used only in tests and we need a fixed value we can test against.
    12345
}

fn get_randomness_seed() -> Vec<u8> {
    // this is an array of u8 of length 8.
    let time_seed = ic_cdk::api::time().to_be_bytes();
    // we need to extend this to an array of size 32 by adding to it an array of size 24 full of 0s.
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
