# Solving VetKD File Sharing Decryption Issue

You've identified a critical issue with the current implementation: when using VetKD with a user's principal as the derivation ID, recipients won't be able to decrypt shared files since the files were encrypted with the owner's principal, not the recipient's.

Let's explore different approaches to solve this problem:

## Option 1: Re-encrypt Files for Each Recipient

The most straightforward approach is to re-encrypt the file for each recipient when sharing.

### How It Would Work:

1. When User A shares a file with User B:
   - Download the file from storage
   - Decrypt it using User A's principal
   - Re-encrypt it using User B's principal
   - Store both versions

### Implementation:

```rust
// Backend share_file function
pub fn share_file(
    state: &mut State,
    caller: Principal,
    sharing_with: Principal,
    file_id: u64,
    recipient_encrypted_content: Vec<u8>, // New encrypted content for recipient
) -> FileSharingResponse {
    // Permission checks...
    
    // Store the recipient-specific encrypted version
    state.file_contents_for_recipients
        .insert((file_id, sharing_with), recipient_encrypted_content);
    
    // Update sharing records
    let file_shares = state
        .file_shares
        .entry(sharing_with)
        .or_insert_with(Vec::new);
    if !file_shares.contains(&file_id) {
        file_shares.push(file_id);
    }
    
    FileSharingResponse::Ok
}

// Modified download_file function
pub fn download_file(
    s: &State,
    file_id: u64,
    chunk_id: u64,
    caller: Principal,
) -> FileDownloadResponse {
    // If caller is the owner, serve original content
    if is_owner(s, caller, file_id) {
        return get_owner_file_data(s, file_id, chunk_id);
    }
    
    // If file is shared with caller, serve their specific encrypted version
    if is_file_shared_with_me(s, file_id, caller) {
        return get_recipient_file_data(s, file_id, chunk_id, caller);
    }
    
    FileDownloadResponse::PermissionError
}
```

**Pros:** Simple to understand and secure
**Cons:** Storage inefficient, requires frontend re-encryption, large files will be slow to share

## Option 2: Use File ID as Derivation ID (with Permission Check)

Instead of using the user's principal, we could use the file ID as the derivation ID, and rely on backend permission checks to control access.

### How It Would Work:

1. Encrypt files using the file ID as derivation ID
2. Check permissions in the backend before serving the file
3. Decrypt using the same file ID

### Implementation:

```rust
// In vetkdCrypto.ts - modified encrypt method
async encrypt(
    data: ArrayBuffer,
    fileId: bigint, // Use file ID instead of principal
): Promise<Uint8Array> {
    // Get public key from the backend
    const publicKeyResponse = await this.actor.vetkd_public_key();
    const publicKey = publicKeyResponse.Ok as Uint8Array;
    
    // Use file ID bytes as derivation ID
    const fileIdBytes = new Uint8Array(
        // Convert bigint to bytes - can use an existing helper library
        convertBigIntToBytes(fileId) 
    );
    
    // Encrypt the data using vetkd IBE with file ID
    return vetkd.IBECiphertext.encrypt(
        publicKey,
        fileIdBytes, // File ID as derivation ID
        new Uint8Array(data),
        window.crypto.getRandomValues(new Uint8Array(32))
    ).serialize();
}

// Decrypt method also uses file ID
async decrypt(
    encryptedData: Uint8Array,
    fileId: bigint, // Use file ID instead of principal
): Promise<Uint8Array> {
    // Similar changes to use fileIdBytes instead of userPrincipalBytes
}
```

**Pros:** Simple, no storage duplication, works with any user who has permission
**Cons:** Less secure if backend permission check is compromised

## Option 3: Use a File-Specific Key Encrypted with VetKD

Use a hybrid approach where each file has a symmetric key, which is then encrypted using VetKD for each user.

### How It Would Work:

1. When uploading a file:
   - Generate a random symmetric key
   - Encrypt the file with this symmetric key
   - Encrypt the symmetric key with VetKD for the owner
   
2. When sharing:
   - Encrypt the symmetric key with VetKD for the recipient
   - Store this encrypted key in the backend

3. When downloading:
   - Get the file and the user-specific encrypted key
   - Decrypt the key using VetKD
   - Use the key to decrypt the file

### Implementation:

```typescript
// Upload process
async uploadFile(file: File) {
    // 1. Generate a random symmetric key
    const symmetricKey = window.crypto.getRandomValues(new Uint8Array(32));
    
    // 2. Encrypt file with symmetric key
    const encryptedData = await encryptWithAES(file, symmetricKey);
    
    // 3. Encrypt symmetric key with VetKD
    const userPrincipal = this.auth.authClient.getIdentity().getPrincipal();
    const encryptedKey = await this.vetkdCryptoService.encryptKey(
        symmetricKey,
        userPrincipal.toUint8Array()
    );
    
    // 4. Upload encrypted file and encrypted key
    const fileId = await this.actor.upload_file_with_key({
        file_content: encryptedData,
        encrypted_key: encryptedKey,
        file_type: file.type,
        name: file.name
    });
    
    return fileId;
}

// Sharing process
async shareFile(fileId: bigint, recipientPrincipal: Principal) {
    // 1. Get the file's symmetric key (sender must be able to decrypt it)
    const encryptedKeyForMe = await this.actor.get_file_key(fileId);
    const symmetricKey = await this.vetkdCryptoService.decryptKey(
        encryptedKeyForMe,
        this.auth.authClient.getIdentity().getPrincipal().toUint8Array()
    );
    
    // 2. Encrypt symmetric key for recipient
    const encryptedKeyForRecipient = await this.vetkdCryptoService.encryptKey(
        symmetricKey,
        recipientPrincipal.toUint8Array()
    );
    
    // 3. Share the re-encrypted key
    await this.actor.share_file(
        recipientPrincipal,
        fileId,
        encryptedKeyForRecipient
    );
}
```

**Pros:** Storage efficient, secure, works well with large files
**Cons:** More complex implementation, still requires key re-encryption but for smaller data

## Option 4: Use a Group-Based Derivation ID

Create a derivation ID that represents a group of users who have access to a file.

### How It Would Work:

1. For each file, create a unique group ID
2. Map this group ID to a list of authorized users in the backend
3. Use the group ID as the derivation ID for encryption/decryption

### Implementation:

```typescript
// When creating a file
async uploadFile(file: File) {
    // 1. Generate a random group ID for this file
    const groupId = crypto.randomUUID();
    
    // 2. Register the group with the backend
    await this.actor.create_access_group(groupId, [
        this.auth.authClient.getIdentity().getPrincipal()
    ]);
    
    // 3. Encrypt using group ID as derivation ID
    const encryptedData = await this.vetkdCryptoService.encrypt(
        file,
        new TextEncoder().encode(groupId)
    );
    
    // 4. Upload file with group ID
    const fileId = await this.actor.upload_file({
        content: encryptedData,
        group_id: groupId,
        file_type: file.type,
        name: file.name
    });
}

// When sharing
async shareFile(fileId: bigint, recipientPrincipal: Principal) {
    // Get the group ID for this file
    const groupId = await this.actor.get_file_group_id(fileId);
    
    // Add recipient to the group
    await this.actor.add_to_access_group(
        groupId, 
        recipientPrincipal
    );
}
```

**Pros:** Clean conceptual model, efficient for many users
**Cons:** Requires additional backend state management

## Recommendation: Option 3 - Hybrid Approach with File-Specific Keys

The hybrid approach with file-specific symmetric keys encrypted via VetKD offers the best balance:

1. **Storage Efficiency**: Only the file's symmetric key needs to be re-encrypted, not the entire file.
2. **Security**: Each file has its own key, and each user gets a unique encrypted version.
3. **Scalability**: Works well even with large files and many recipients.
4. **Minimal Changes**: Your existing VetKD infrastructure can be reused to encrypt/decrypt keys.

### Implementation Changes Required:

1. **Backend State Structure**:
   - Add a map to store user-specific encrypted keys: `file_keys: BTreeMap<(FileId, Principal), Vec<u8>>`

2. **Backend API**:
   - Add endpoint to store/retrieve file keys
   - Modify sharing endpoint to store recipient-specific keys

3. **Frontend Flow**:
   - Modify upload process to use symmetric encryption for the file and VetKD for the key
   - Modify sharing to re-encrypt the key (not the file) for recipients
   - Modify download to first get the encrypted key, then the file

This approach leverages the strengths of both symmetric encryption (efficiency for large data) and VetKD (secure key management tied to user identity).

Would you like me to elaborate on any of these approaches or suggest implementation details for your preferred solution?


# Option 1: Re-encrypt Files for Each Recipient - Implementation

I'll walk through the specific files and code changes needed to implement Option 1 (re-encrypting files for each recipient).

## 1. Backend Changes

### A. Update `lib.rs` to Add New State Storage

```rust
// In ic-docutrack/backend/src/lib.rs

pub struct State {
    // Existing fields
    file_count: u64,
    pub users: BTreeMap<Principal, User>,
    pub file_data: BTreeMap<u64, File>,
    pub file_alias_index: BTreeMap<String, u64>,
    pub file_owners: BTreeMap<Principal, Vec<u64>>,
    pub file_shares: BTreeMap<Principal, Vec<u64>>,
    
    // NEW FIELD: Store recipient-specific file content
    #[serde(skip, default = "init_recipient_file_contents")]
    pub recipient_file_contents: StableBTreeMap<(FileId, Principal, ChunkId), Vec<u8>, Memory>,
    
    // Existing fields
    #[serde(skip, default = "init_file_contents")]
    pub file_contents: StableBTreeMap<(FileId, ChunkId), Vec<u8>, Memory>,
    #[serde(skip, default = "init_alias_generator")]
    alias_generator: AliasGenerator,
}

// Add a new memory initialization function
fn init_recipient_file_contents() -> StableBTreeMap<(FileId, Principal, ChunkId), Vec<u8>, Memory> {
    StableBTreeMap::init(crate::memory::get_recipient_file_contents_memory())
}
```

### B. Modify `memory.rs` to Add New Memory for Recipient Files

```rust
// In ic-docutrack/backend/src/memory.rs

// Add a new memory ID for recipient file contents
const RECIPIENT_FILE_CONTENTS: MemoryId = MemoryId::new(2);

// Add a function to get this memory
pub fn get_recipient_file_contents_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(RECIPIENT_FILE_CONTENTS))
}
```

### C. Update `share_file.rs` to Store Recipient-Specific File Content

```rust
// In ic-docutrack/backend/src/api/share_file.rs

// Modify the share_file function to accept re-encrypted content
pub fn share_file(
    state: &mut State,
    caller: Principal,
    sharing_with: Principal,
    file_id: u64,
    recipient_encrypted_chunks: Vec<(u64, Vec<u8>)>, // Format: [(chunk_id, chunk_data), ...]
) -> FileSharingResponse {
    if !can_share(state, caller, file_id) {
        FileSharingResponse::PermissionError
    } else {
        let file = state.file_data.get(&file_id).unwrap();
        match &file.content {
            FileContent::Pending { .. } | FileContent::PartiallyUploaded { .. } => {
                FileSharingResponse::PendingError
            }
            FileContent::Uploaded { num_chunks, .. } => {
                // Validate that all chunks are provided
                let received_chunks = recipient_encrypted_chunks.len() as u64;
                if received_chunks != *num_chunks {
                    return FileSharingResponse::ChunkCountMismatch {
                        expected: *num_chunks,
                        received: received_chunks,
                    };
                }
                
                // Store the recipient-specific chunks
                for (chunk_id, chunk_data) in recipient_encrypted_chunks {
                    state.recipient_file_contents.insert(
                        (file_id, sharing_with, chunk_id),
                        chunk_data
                    );
                }
                
                // Add to shared files list
                let file_shares = state
                    .file_shares
                    .entry(sharing_with)
                    .or_insert_with(Vec::new);

                if !file_shares.contains(&file_id) {
                    file_shares.push(file_id);
                }

                FileSharingResponse::Ok
            }
        }
    }
}
```

### D. Update `download_file.rs` to Serve Recipient-Specific Content

```rust
// In ic-docutrack/backend/src/api/download_file.rs

// Modify the download_file function to check for recipient-specific content
pub fn download_file(
    s: &State,
    file_id: u64,
    chunk_id: u64,
    caller: Principal,
) -> FileDownloadResponse {
    // Check if this file is owned by the caller
    let is_owner = match s.file_owners.get(&caller) {
        Some(files) => files.contains(&file_id),
        None => false,
    };
    
    // Check if this file is shared with the caller
    let is_shared = is_file_shared_with_me(s, file_id, caller);
    
    if !is_owner && !is_shared {
        return FileDownloadResponse::PermissionError;
    }
    
    // Get file metadata
    let this_file = match s.file_data.get(&file_id) {
        Some(file) => file,
        None => return FileDownloadResponse::NotFoundFile,
    };
    
    match &this_file.content {
        FileContent::Pending { .. } | FileContent::PartiallyUploaded { .. } => {
            FileDownloadResponse::NotUploadedFile
        }
        FileContent::Uploaded { file_type, num_chunks, .. } => {
            // Check if there's recipient-specific content (for shared files)
            let contents = if !is_owner && is_shared {
                // Use recipient-specific content
                match s.recipient_file_contents.get(&(file_id, caller, chunk_id)) {
                    Some(content) => content,
                    None => return FileDownloadResponse::NotFoundFile,
                }
            } else {
                // Use original content (for owner)
                match s.file_contents.get(&(file_id, chunk_id)) {
                    Some(content) => content,
                    None => return FileDownloadResponse::NotFoundFile,
                }
            };
            
            FileDownloadResponse::FoundFile(FileData {
                contents,
                file_type: file_type.clone(),
                num_chunks: *num_chunks,
            })
        }
    }
}
```

### E. Update `FileSharingResponse` in `lib.rs`

```rust
// In ic-docutrack/backend/src/lib.rs

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum FileSharingResponse {
    #[serde(rename = "pending_error")]
    PendingError,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "ok")]
    Ok,
    // New variant
    #[serde(rename = "chunk_count_mismatch")]
    ChunkCountMismatch {
        expected: u64,
        received: u64,
    },
}
```

### F. Update `main.rs` Entry Points

```rust
// In ic-docutrack/backend/src/main.rs

#[update]
fn share_file(
    user_id: Principal,
    file_id: u64,
    recipient_encrypted_chunks: Vec<(u64, Vec<u8>)>,
) -> FileSharingResponse {
    with_state_mut(|s| backend::api::share_file(
        s, 
        caller(), 
        user_id, 
        file_id, 
        recipient_encrypted_chunks
    ))
}
```

### G. Update Candid Interface in `service.did`

```did
// In ic-docutrack/backend/service.did

type chunk_data = record {
  chunk_id: nat64;
  data: blob;
};

type share_file_response = variant {
  permission_error;
  pending_error;
  chunk_count_mismatch: record { expected: nat64; received: nat64; };
  ok;
};

// Update the share_file method signature
share_file: (user_id: principal, file_id: file_id, recipient_encrypted_chunks: vec chunk_data) -> (share_file_response);
```

## 2. Frontend Changes

### A. Update `vetkdCrypto.ts` with Re-encryption Helper

```typescript
// In ic-docutrack/frontend/src/frontend/src/lib/vetkeys/vetkdCrypto.ts

// Add a method to re-encrypt for recipient
async reencryptForRecipient(
  encryptedData: Uint8Array,
  ownerPrincipalBytes: Uint8Array,
  recipientPrincipalBytes: Uint8Array
): Promise<Uint8Array> {
  // 1. Decrypt the data using owner's principal
  const decryptedData = await this.decrypt(
    encryptedData,
    ownerPrincipalBytes
  );
  
  // 2. Re-encrypt for recipient
  return await this.encrypt(
    decryptedData.buffer,
    recipientPrincipalBytes
  );
}
```

### B. Update `ShareModal.svelte` to Re-encrypt File

```svelte
<!-- In ic-docutrack/frontend/src/frontend/src/lib/components/ShareModal.svelte -->

<script lang="ts">
  // Existing imports...
  
  async function saveShare() {
    if (!enumIs(fileData.file_status, "uploaded")) {
      return;
    }

    loading = true;
    error = "";
    
    // Get owner principal for decryption
    const ownerPrincipal = auth.authClient.getIdentity().getPrincipal();
    const ownerPrincipalBytes = ownerPrincipal.toUint8Array();
    
    for (let i = 0; i < newSharedWith.length; i++) {
      const recipientPrincipal = newSharedWith[i].ic_principal;
      
      // Skip if already shared with this user
      const alreadyShared = oldSharedWith.some(
        (user) => user.ic_principal.compareTo(recipientPrincipal) === "eq",
      );
      if (alreadyShared) continue;
      
      try {
        // Re-encrypt each chunk of the file for the recipient
        const recipientPrincipalBytes = recipientPrincipal.toUint8Array();
        const reencryptedChunks = [];
        
        // Get the number of chunks from file metadata
        const numChunks = Number(fileData.file_status.uploaded.num_chunks || 1);
        
        for (let chunkId = 0; chunkId < numChunks; chunkId++) {
          // Download the chunk
          const response = await auth.actor.download_file(
            fileData.file_id,
            BigInt(chunkId)
          );
          
          if (!enumIs(response, "found_file")) {
            throw new Error(`Couldn't download chunk ${chunkId}`);
          }
          
          // Re-encrypt this chunk for the recipient
          const reencryptedChunk = await vetkdCryptoService.reencryptForRecipient(
            response.found_file.contents,
            ownerPrincipalBytes,
            recipientPrincipalBytes
          );
          
          // Add to the list of re-encrypted chunks
          reencryptedChunks.push([BigInt(chunkId), reencryptedChunk]);
        }
        
        // Share the file with re-encrypted chunks
        const shareResult = await auth.actor.share_file(
          recipientPrincipal,
          fileData.file_id,
          reencryptedChunks
        );
        
        if (enumIs(shareResult, "permission_error")) {
          throw new Error("Permission denied to share file");
        }
        if (enumIs(shareResult, "chunk_count_mismatch")) {
          throw new Error(`Chunk count mismatch: expected ${shareResult.chunk_count_mismatch.expected}, got ${shareResult.chunk_count_mismatch.received}`);
        }
      } catch (err) {
        error = `Error: could not share file with ${newSharedWith[i].username}: ${err.message}`;
        loading = false;
        return;
      }
    }
    
    // Handle revoking shares (rest of the function)
    // ...
  }
</script>
```

### C. Update Declaration Types

After making these changes, you'll need to regenerate the TypeScript declarations for your backend canister interface. This typically happens automatically during the build process with `dfx build` or `dfx deploy`.

## Summary of Changes

1. **Backend Storage**:
   - Add storage for recipient-specific file chunks
   - Use a three-part key: (file_id, recipient_principal, chunk_id)

2. **Sharing Logic**:
   - Accept re-encrypted chunks during sharing
   - Store them in recipient-specific storage
   - Update file shares index

3. **Download Logic**:
   - Check if user is owner or recipient
   - Serve appropriate content (original for owner, re-encrypted for recipient)

4. **Frontend Changes**:
   - Add helper to re-encrypt data for recipients
   - Modify sharing flow to download, decrypt, re-encrypt, and upload chunks

This implementation offers a straightforward approach where each recipient gets their own encrypted version of the file, allowing them to decrypt it using their own principal as the derivation ID. The tradeoff is increased storage use, but it preserves the security model of identity-based encryption.

## Important Note

For large files, this approach might be slow during sharing, as it requires downloading and re-uploading each chunk. Consider adding progress indicators during the sharing process to let users know what's happening.

Would you like me to explain any part of this implementation in more detail?

