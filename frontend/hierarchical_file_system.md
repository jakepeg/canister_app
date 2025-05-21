# Backend Refactor: Hierarchical File System

Version: 1.1 (Post-Initial Refactor for Hierarchy)
Date: 2025-05-20 (Assumed date of this summary)

## 1. Overview & Goal

The backend was refactored from a flat file management system to a hierarchical (tree-like) structure. Users can now create folders, organize files within them, and navigate this structure. This document summarizes the key changes made to backend data structures and API logic.

## 2. Core Data Structure Changes (lib.rs)
### 2.1. Central Item Metadata

Before:

- Separate FileMetadata struct for files.
- File content status was managed by a FileContent enum (Pending, PartiallyUploaded, Uploaded) within a File struct.
- State primarily held file_data: BTreeMap<u64, File>.

After:

- Introduced ItemId = u64 as a general identifier for both files and folders.
- Introduced ItemType = enum { File, Folder }.
- Introduced a unified ItemMetadata struct:

```rust
pub struct ItemMetadata {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>, // None for root items
    pub owner_principal: Principal,
    pub created_at: u64,
    pub modified_at: u64,
    // File-specific (Option<T>):
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub num_chunks: Option<u64>,
}
```

- The main state now holds items: BTreeMap<ItemId, ItemMetadata>.
- The old File struct and FileContent enum are deprecated/removed. File content status (pending, partial, complete) is now derived from ItemMetadata fields (e.g., content_type.is_none(), num_chunks vs actual chunks in file_contents).

### 2.2. Public Data Transfer Object (DTO) for Frontend

Before: PublicFileMetadata (focused on files and included legacy FileStatus).

After: Introduced PublicItemMetadata:

```rust
pub struct PublicItemMetadata {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
    pub parent_id: Option<ItemId>,
    pub modified_at: u64,
    pub size: Option<u64>, // Null for folders
}
```

This is the primary DTO for sending item lists to the frontend.

### 2.3. State Management (State struct in lib.rs)

Before:

- file_count: u64
- file_data: BTreeMap<u64, File>
- file_owners: BTreeMap<Principal, Vec<u64>>
- file_shares: BTreeMap<Principal, Vec<u64>>
- file_contents: StableBTreeMap<(u64, ChunkId), Vec<u8>>

After:

- item_id_counter: ItemId (renamed from file_count)
- items: BTreeMap<ItemId, ItemMetadata> (replaces file_data)
- item_owners: BTreeMap<Principal, Vec<ItemId>> (replaces file_owners)
- item_shares: BTreeMap<Principal, Vec<ItemId>> (replaces file_shares)
- file_contents: StableBTreeMap<(ItemId, ChunkId), Vec<u8>> (key now uses ItemId)
- file_alias_index: Value changed from u64 to ItemId.
- group_files (in RequestGroup context): Values changed to Vec<ItemId>.

## 3. API Endpoint Changes (main.rs and api/*.rs)
### 3.1. New Endpoints

create_folder(name: String, parent_id: Option<ItemId>) -> Result<PublicItemMetadata, String>

- Purpose: Creates a new ItemMetadata with item_type: ItemType::Folder.
- Logic: Assigns ID, sets owner, timestamps, parent_id. Stores in state.items and state.item_owners.

list_folder_contents(folder_id: Option<ItemId>) -> Result<Vec<PublicItemMetadata>, String> (Query)

- Purpose: Returns PublicItemMetadata for direct children of folder_id (or root if folder_id is None).
- Logic: Filters state.items based on parent_id and caller permissions.

move_item(item_id: ItemId, new_parent_id: Option<ItemId>) -> Result<(), String>

- Purpose: Changes the parent_id of an existing item (file or folder).
- Logic: Updates ItemMetadata.parent_id and modified_at. Includes permission checks and potentially cycle detection for folders.

### 3.2. Modified Existing Endpoints

Upload Endpoints (e.g., upload_file_atomic, upload_file_to_item):

- upload_file_atomic Request (UploadFileAtomicDirectRequest):
  - Now includes parent_id: Option<ItemId>.
  - Creates an ItemMetadata of type File with the given parent_id.

- upload_content_to_item (was upload_file) for Alias-Based Uploads:
  - Takes item_id (of the pending item).
  - Updates the existing ItemMetadata (e.g., sets content_type, size, num_chunks, modified_at).

- upload_chunk_continue:
  - Takes item_id.
  - Updates modified_at of the ItemMetadata and increments its size (if applicable).

- request_file(request_name: String, parent_id: Option<ItemId>) -> Result<String, String>:
  - Now takes parent_id.
  - Creates a placeholder ItemMetadata of item_type: File with the specified parent_id. File-specific fields like content_type, size, num_chunks are None initially.
  - Returns an alias pointing to this new item_id.

Item Manipulation Endpoints (were "file" manipulation):

- rename_item (was rename_file):
  - Operates on item_id.
  - Updates ItemMetadata.name and modified_at.
  - Works for both files and folders.

- delete_item (was delete_file):
  - Operates on item_id.
  - If ItemType::File: Deletes metadata from state.items, content from state.file_contents, and cleans up owner/share/alias/group references.
  - If ItemType::Folder: Currently implemented to only allow deletion if the folder is empty. Recursive deletion is a future consideration. Also cleans up metadata and references.

- share_item / revoke_item_share (were share_file / revoke_share):
  - Operate on item_id.
  - Update state.item_shares.
  - Update ItemMetadata.modified_at.
  - Files can only be shared if fully uploaded (derived status). Folders can be shared.

Listing/Query Endpoints:

- get_items_shared_with_me (was get_shared_files): Returns Vec<PublicItemMetadata>.
- get_my_pending_requests (was get_requests): Clarified purpose. Filters state.items for items owned by the caller that are ItemType::File, have an associated alias in state.file_alias_index, and content_type is None. Returns Vec<PublicItemMetadata>.
- get_alias_info: Takes an alias, returns Result<AliasInfoForUpload, GetAliasInfoError>. AliasInfoForUpload now contains item_id. Checks if the target item is ItemType::File.

- download_file_chunk (was download_file):
  - Takes item_id and chunk_id.
  - Checks permissions.
  - Errors if item is a folder or not fully uploaded.
  - Returns DownloadChunkResponse containing chunk content and original file info.

"Request Group" Related (Marked _legacy in .did):

- multi_request_legacy:
  - Creates a new "group" as an ItemMetadata of ItemType::Folder (typically at root, parent_id: None).
  - Creates multiple placeholder ItemMetadata entries (type File) with their parent_id set to the group folder's ID. Each gets an individual alias.
  - The group_alias in the response refers to the folder.

- get_request_groups_legacy: Lists these "group folders" and their child PublicItemMetadata.
- get_group_by_alias_legacy: Takes a group (folder) alias, returns info about the folder and a list of FileInfoForUpload (file item_id, name, individual alias) for items within it.

VetKD Endpoints:

- vetkd_encrypted_key: Parameter changed from file_id: Option<u64> to item_id: Option<ItemId>. Internal logic now fetches owner_principal from state.items.get(&id).item_meta.owner_principal.

### 3.3. Candid Interface (.did file)

- Reflects all new types (item_id, item_type, public_item_metadata, new request/response DTOs).
- Reflects new endpoint signatures (create_folder, list_folder_contents, move_item, etc.).
- Reflects updated signatures for existing endpoints (e.g., item_id parameters, Result return types).
- Legacy types/endpoints related to the old file/group system are suffixed with _legacy to indicate they may be phased out or require further refactoring.

## 4. Impact on File Status Logic

Before: Backend explicitly stored and returned FileStatus enum (pending, partially_uploaded, uploaded).

After:

- No single explicit "status" enum is stored per item in ItemMetadata or returned in PublicItemMetadata.
- Status is derived based on ItemMetadata fields and context:
  - Item Type: item.item_type (File or Folder).
  - File - Pending Alias Upload: item_type == File, content_type is None, and an alias exists for its item_id in state.file_alias_index.
  - File - Upload In Progress/Partially Uploaded: item_type == File, content_type is Some, num_chunks is Some(N), but state.num_chunks_uploaded(item_id) < N.
  - File - Upload Complete/Usable: item_type == File, content_type is Some, num_chunks is Some(N), and state.num_chunks_uploaded(item_id) == N.
  - Folder - Exists: item_type == Folder.

- Frontend Implication: The frontend must now use these derived logic rules to determine how to display an item and what actions are available, instead of directly switching on a file_status enum.
- Backend operations (like download, share) will return errors if an item is not in an appropriate state (e.g., NotUploadedFile from download_file_chunk, PendingError from share_item).

## 5. Data Migration (Upgrade Path)

If upgrading an existing canister with data, post_upgrade in upgrade.rs would need a migration function:

- To convert old State.file_data (containing File structs) into the new State.items (containing ItemMetadata).
- All old items would become ItemType::File with parent_id: None.
- Timestamps, owner, and file-specific info would be mapped from old structures to ItemMetadata.
- file_owners -> item_owners, file_shares -> item_shares.
- Keys in file_contents map would need to be updated from (old_file_id, ...) to (item_id, ...).

This summary should provide a good foundation for understanding the backend changes. The key is the shift to a unified ItemMetadata with parent_id for hierarchy and the derivation of status rather than explicit storage for all cases.
