# Vetkeys Demystified: Encrypting Data in Your ICP App

If you're building an app on the Internet Computer (ICP) that requires
encrypting user data, you've likely heard about vetkeys. They offer a powerful,
decentralized way to manage encryption keys without relying on centralized key
servers. While the official docs are getting better, figuring out the practical
implementation details can still involve some digging and experimentation
(shoutout to Kristofer and his ETH vetkey demo app for inspiration!).

This guide is for developers looking to implement vetkey encryption in their ICP
applications. I'll walk through the core concepts and provide practical code
examples based on my experience building Canister Cloud, a full-stack
application using Rust for the backend and Svelte for the frontend. We'll focus
on the fundamental flow: encrypting data for storage and decrypting it upon
retrieval.

Most of the setup and implementation details will be useful regardless of your
specific stack, though you might need to adapt data types and logic flows
slightly. My goal is to make vetkeys more accessible by highlighting the key
steps and providing concrete examples.

## What You'll Need

- An ICP full-stack project (dfx).
- A backend canister (Rust examples provided).
- A frontend framework (Svelte examples provided).
- The `ic-vetkd-utils` frontend package.
- Basic familiarity with ICP development concepts (canisters, principals,
  Candid).

## Core Concepts (Simplified)

Vetkeys utilize **Identity-Based Encryption (IBE)**. Instead of generating a
traditional public/private key pair for _each_ piece of data, IBE allows
deriving unique keys based on:

1. **A Master Public Key:** Provided by the vetkey system on ICP.
2. **A Derivation ID:** A unique identifier, often the user's `Principal`, which
   ties the derived key to a specific identity.

**The Basic Flow:**

- **Encryption (Frontend):**
  1. Get the vetkd system's master public key from your backend.
  2. Use the user's `Principal` (as bytes) as the derivation ID.
  3. Use the `ic-vetkd-utils` library to encrypt the data using the master
     public key and the derivation ID.
  4. Store the resulting encrypted blob.
- **Decryption (Frontend):**
  1. To decrypt data encrypted with a specific derivation ID (e.g., the user's
     Principal), you need the corresponding _derived private key_.
  2. You can't get this key directly. Instead, you generate a temporary
     **transport key pair** (public/private) in the frontend.
  3. You send the _transport public key_ and the _derivation ID_ (user's
     Principal) to your backend.
  4. The backend asks the vetkd system API to derive the private key for that
     derivation ID and encrypt it using the provided _transport public key_.
  5. The backend sends this encrypted derived key back to the frontend.
  6. The frontend uses its _transport private key_ to decrypt the derived key.
  7. Finally, the frontend uses this decrypted derived key to decrypt the actual
     data blob.

This might sound complex, but it ensures that only the user associated with the
derivation ID can ultimately obtain the necessary key to decrypt their data,
facilitated securely by the backend and the vetkd system.

## Backend Setup

### 1. Configure `dfx.json`

You need to tell `dfx` about the vetkd system canister. Add the following to
your `canisters` object in `dfx.json`:

```json
"vetkd_system_api": {
  "candid": "./vetkeys/chainkey_testing_canister.did", // Adjust path if needed
  "type": "custom",
  "wasm": "./vetkeys/chainkey_testing_canister.wasm", // Adjust path if needed
  "declarations": {
    "output": "src/declarations/vetkd_system_api" // Or your preferred declarations path
  }
},
```

_(Make sure you have the `.did` and `.wasm` files for the vetkd system API,
often provided in ICP examples or starter projects)._

### 2. Define Vetkey Methods in `service.did`

Expose the necessary vetkey methods in your backend canister's Candid interface
(`.did` file):

```did
type VetkdEncryptedKeyResponse = variant {
  Ok : blob;
  Err : text;
};

type VetkdPublicKeyResponse = variant {
  Ok : blob;
  Err : text;
};

service : {
  // ... other methods
  vetkd_public_key : () -> (VetkdPublicKeyResponse);
  vetkd_encrypted_key : (blob, opt nat64) -> (VetkdEncryptedKeyResponse); // blob is transport_pubkey, opt nat64 is file_id
  // ... other methods
}
```

### 3. Implement Backend Rust Functions

You need Rust functions in your backend canister to interact with the vetkd
system API.

**a) Getting the Master Public Key:**

This function fetches the main vetkd public key used for encryption.

```rust
// backend/src/vetkd/controller/vetkd_public_key.rs
use crate::declarations::vetkd_system_api::{
    vetkd_system_api, VetkdCurve, VetkdPublicKeyArgs, VetkdPublicKeyArgsKeyId,
};
use ic_cdk::update;

#[update]
async fn vetkd_public_key() -> Result<Vec<u8>, String> {
    let args = VetkdPublicKeyArgs {
        key_id: VetkdPublicKeyArgsKeyId {
            // IMPORTANT: Use a production key ID in a real application!
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![], // Not typically needed for basic IBE
        canister_id: None,       // Use the canister's own ID
    };

    // Call the system API
    let (result,) = vetkd_system_api.vetkd_public_key(args).await.map_err(|e| format!("vetkd_public_key failed: {:?}", e))?;

    Ok(result.public_key.to_vec())
}
```

_Note:_ This example uses `"insecure_test_key_1"`. For production, you'll need
to use a proper key ID like `"key_1"` or `"test_key_1"` depending on the network
and desired security level.

**b) Getting the Derived Encrypted Key:**

This function handles the frontend's request for the derived key needed for
decryption. It takes the frontend's transport public key and derives the key
based on the file owner's principal (looked up via `file_id`).

```rust
// backend/src/vetkd/controller/vetkd_encrypted_key.rs
use crate::declarations::vetkd_system_api::{
    vetkd_system_api, VetkdCurve, VetkdDeriveEncryptedKeyArgs, VetkdDeriveEncryptedKeyArgsKeyId,
};
use crate::with_state; // Assuming you have a way to access shared state
use ic_cdk::update;
use serde_bytes::ByteBuf;

#[update]
async fn vetkd_encrypted_key(
    encryption_public_key: Vec<u8>, // Frontend's transport public key
    file_id: Option<u64>,           // ID to find the owner principal
) -> Result<Vec<u8>, String> {

    // Determine the derivation ID (owner's principal)
    let derivation_id_bytes = if let Some(id) = file_id {
        // Look up the file's owner principal from your state/metadata
        with_state(|state| {
            state
                .file_data // Assuming file_data map exists in your state
                .get(&id)
                .map(|file| file.metadata.requester_principal.as_slice().to_vec()) // Adjust field access as needed
                .ok_or_else(|| format!("File not found for ID: {}", id))
        })?
    } else {
        // Fallback or error if file_id is required for decryption context
        // For simplicity, let's assume file_id is always provided for decryption
         return Err("File ID is required to determine the correct derivation ID".to_string());
        // Alternatively, use caller if appropriate for your logic:
        // ic_cdk::api::caller().as_slice().to_vec()
    };

    let args = VetkdDeriveEncryptedKeyArgs {
        key_id: VetkdDeriveEncryptedKeyArgsKeyId {
            // IMPORTANT: Use the same key ID as in vetkd_public_key!
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![],
        derivation_id: ByteBuf::from(derivation_id_bytes), // Owner's principal
        encryption_public_key: ByteBuf::from(encryption_public_key), // Frontend's transport key
    };

    // Call the system API
    let (result,) = vetkd_system_api
        .vetkd_derive_encrypted_key(args)
        .await
        .map_err(|e| format!("vetkd_derive_encrypted_key failed: {:?}", e))?;

    Ok(result.encrypted_key.to_vec())
}
```

Remember to expose these Rust functions in your `backend/src/lib.rs` or
`main.rs` using `#[ic_cdk::update]` macros so they are callable from the
frontend.

## Frontend Setup

### 1. Install `ic-vetkd-utils`

Add the package to your `package.json`. You might be using a `.tgz` file if it's
not yet published or if you're using a specific version:

```json
// package.json (dependencies)
"ic-vetkd-utils": "file:./ic-vetkd-utils-0.1.0.tgz", // Or the appropriate version/source
```

Run `npm install` or `pnpm install`.

### 2. Create a Vetkey Service

It's good practice to encapsulate the vetkey logic in a dedicated service class.

```typescript
// frontend/src/frontend/src/lib/vetkeys/vetkdCrypto.ts
import * as vetkd from "ic-vetkd-utils";
import type { ActorType } from "$lib/shared/actor"; // Your backend actor type

export class VetkdCryptoService {
    constructor(private actor: ActorType) {}

    // Encryption method (details below)
    async encrypt(
        data: ArrayBuffer,
        userPrincipalBytes: Uint8Array,
    ): Promise<Uint8Array> {
        // ... implementation ...
    }

    // Decryption method (details below)
    async decrypt(
        encryptedData: Uint8Array,
        userPrincipalBytes: Uint8Array, // Current user's principal
        fileId: bigint, // Needed to get the correct derived key from backend
    ): Promise<Uint8Array> {
        // ... implementation ...
    }
}
```

## Implementation: Encrypting & Uploading

Here's the step-by-step flow for encrypting a file before uploading:

**Step 1: Get Vetkd Public Key**

Your frontend needs the master public key from the backend. This is usually done
once or periodically.

```typescript
// Inside an async function where you have access to your actor
const publicKeyResponse = await this.actor.vetkd_public_key();
if (!publicKeyResponse || "Err" in publicKeyResponse) {
    throw new Error(
        "Error getting public key: " +
            ("Err" in publicKeyResponse
                ? publicKeyResponse.Err
                : "empty response"),
    );
}
const vetkdPublicKey = publicKeyResponse.Ok as Uint8Array;
```

**Step 2: Prepare Data and Derivation ID**

Get the file content as an `ArrayBuffer` and the current user's principal as
`Uint8Array`.

```typescript
const fileBytes: ArrayBuffer = await file.arrayBuffer(); // file is a File object
const userPrincipalBytes: Uint8Array = auth.authClient.getIdentity()
    .getPrincipal().toUint8Array(); // Assuming 'auth' service
```

**Step 3: Encrypt using `ic-vetkd-utils`**

Use the `VetkdCryptoService.encrypt` method, which wraps the `ic-vetkd-utils`
library call:

```typescript
// frontend/src/frontend/src/lib/vetkeys/vetkdCrypto.ts (encrypt method)
async encrypt(
  data: ArrayBuffer,
  userPrincipalBytes: Uint8Array, // This is our Derivation ID
): Promise<Uint8Array> {
  try {
    // 1. Get vetkd public key (as shown in Step 1 above)
    const publicKeyResponse = await this.actor.vetkd_public_key();
    // ... (handle errors) ...
    const vetkdPublicKey = publicKeyResponse.Ok as Uint8Array;

    // 2. Generate a random seed (required by the library)
    const seed = window.crypto.getRandomValues(new Uint8Array(32));

    // 3. Ensure data is Uint8Array
    const encodedMessage = new Uint8Array(data);

    // 4. Encrypt using IBE
    const encryptedData = vetkd.IBECiphertext.encrypt(
      vetkdPublicKey,
      userPrincipalBytes, // Derivation ID
      encodedMessage,
      seed,
    );

    // 5. Serialize the result for storage/transport
    return encryptedData.serialize();
  } catch (error) {
    console.error("Encryption error:", error);
    throw error;
  }
}
```

**Step 4: Upload Encrypted Blob**

Your upload service takes the `encryptedData` (the `Uint8Array` returned by
`encrypt`) and uploads it, likely in chunks.

```typescript
// frontend/src/frontend/src/lib/services/upload.ts (simplified relevant parts)
import { VetkdCryptoService } from "../vetkeys/vetkdCrypto";
// ... other imports ...
export const CHUNK_SIZE = 2_000_000;

export class UploadService {
  // ... constructor, auth state ...
  private vetkdCryptoService: VetkdCryptoService;

  async uploadFile({ file, ... }: { file: File, ... }) {
    // ... get userPrincipalBytes ...
    const fileBytes = await file.arrayBuffer();

    // Encrypt the data
    const encryptedData = await this.vetkdCryptoService.encrypt(
      fileBytes,
      userPrincipalBytes,
    );

    // Chunk and upload the encryptedData
    const numChunks = Math.ceil(encryptedData.length / CHUNK_SIZE);
    const firstChunk = encryptedData.subarray(0, CHUNK_SIZE);

    // Call backend to start upload (atomic or first chunk)
    const fileId = await this.auth.actor.upload_file_atomic({ // Or upload_file
      content: firstChunk,
      name: file.name,
      file_type: file.type,
      num_chunks: BigInt(numChunks),
    });

    // Upload remaining chunks in parallel
    await this.uploadChunks(encryptedData, fileId, onChunkUploaded);

    // ... handle completion/errors ...
  }

  private async uploadChunks(content: Uint8Array, fileId: bigint, ...) {
     // ... logic to upload chunks from index 1 onwards using upload_file_continue ...
  }
}
```

## Implementation: Downloading & Decrypting

Now, the reverse process:

**Step 1: Download Encrypted Blob**

Fetch all the encrypted chunks from the backend and merge them into a single
`Uint8Array`.

```typescript
// frontend/src/frontend/src/lib/services/decrypt.ts (simplified relevant parts)
export class DecryptService {
  // ... constructor, auth state ...

  async decryptFile({ fileId }: { fileId: bigint }): Promise<...> {
    // ... get file metadata ...

    // Download chunk 0
    let downloadResponse = await this.auth.actor.download_file(fileId, 0n);
    // ... handle errors ...
    let mergedEncryptedData = downloadResponse.found_file.contents;
    const totalChunks = Number(downloadResponse.found_file.num_chunks);

    // Download remaining chunks (1 to n-1)
    for (let i = 1; i < totalChunks; i++) {
      const chunkResponse = await this.auth.actor.download_file(fileId, BigInt(i));
      // ... handle errors ...
      const chunkData = chunkResponse.found_file.contents;

      // Merge chunks
      const temp = new Uint8Array(mergedEncryptedData.length + chunkData.length);
      temp.set(mergedEncryptedData, 0);
      temp.set(chunkData, mergedEncryptedData.length);
      mergedEncryptedData = temp;
    }

    // Now 'mergedEncryptedData' holds the full encrypted file content
    // ... proceed to decryption ...
  }
}
```

**Step 2: Generate Ephemeral Transport Keys**

Create a temporary key pair in the frontend.

```typescript
// Inside VetkdCryptoService.decrypt or where decryption is initiated
const seed = window.crypto.getRandomValues(new Uint8Array(32));
const transportSecretKey = new vetkd.TransportSecretKey(seed);
const transportPublicKeyBytes = transportSecretKey.public_key();
```

**Step 3: Request Derived Key from Backend**

Call the backend `vetkd_encrypted_key` endpoint, passing the _transport public
key_ and the `fileId` (so the backend knows which owner's principal to use for
derivation).

```typescript
// Inside VetkdCryptoService.decrypt
const privateKeyResponse = await this.actor.vetkd_encrypted_key(
    transportPublicKeyBytes,
    [fileId], // Pass fileId as Option<nat64>
);
// ... handle errors ...
const encryptedDerivedKey = privateKeyResponse.Ok as Uint8Array;
```

**Step 4: Decrypt Derived Key**

Use the _transport private key_ to decrypt the response from the backend. You
also need the master vetkd public key and the _original derivation ID_ (the
user's principal in this case) used for encryption.

```typescript
// Inside VetkdCryptoService.decrypt (simplified for non-shared files)

// Get vetkd public key (if not already available)
const publicKeyResponse = await this.actor.vetkd_public_key();
// ... handle errors ...
const vetkdPublicKey = publicKeyResponse.Ok as Uint8Array;

// Get current user's principal (used as derivation ID during encryption)
const userPrincipalBytes = this.auth.authClient.getIdentity().getPrincipal()
    .toUint8Array();

// Decrypt the key received from the backend
const decryptedDerivedKey = transportSecretKey.decrypt(
    encryptedDerivedKey,
    vetkdPublicKey,
    userPrincipalBytes, // The derivation ID used to encrypt
);
```

_(Note: The actual `decrypt` code in your repo handles shared files by
potentially using the owner's principal here. For this guide, we assume the user
is decrypting their own file)._

**Step 5: Decrypt Data**

Finally, use the `decryptedDerivedKey` to decrypt the actual file content.

```typescript
// Inside VetkdCryptoService.decrypt
const ibeCiphertext = vetkd.IBECiphertext.deserialize(mergedEncryptedData);
const decryptedDataBytes = ibeCiphertext.decrypt(decryptedDerivedKey);

// decryptedDataBytes is the original file content as Uint8Array
return decryptedDataBytes;
```

The `DecryptService` would call this `decrypt` method after downloading the
chunks.

```typescript
// frontend/src/frontend/src/lib/services/decrypt.ts (calling decrypt)
// ... after merging chunks into mergedEncryptedData ...

try {
    const userPrincipalBytes = this.auth.authClient.getIdentity().getPrincipal()
        .toUint8Array();

    // Decrypt using the service
    const decryptedData = await this.vetkdCryptoService.decrypt(
        mergedEncryptedData as Uint8Array,
        userPrincipalBytes,
        fileId,
    );

    // Return the decrypted content
    return {
        // ... name, dataType, etc. ...
        contents: decryptedData.buffer as ArrayBuffer,
    };
} catch (err) {
    console.error("Decryption failed:", err);
    // Handle decryption error
}
```

## Key Considerations & Tips

- **Experimental:** Vetkeys are still somewhat experimental. APIs and libraries
  might change. Keep dependencies updated and test thoroughly.
- **Test Keys:** Remember to switch from test key IDs (like
  `"insecure_test_key_1"`) to appropriate production key IDs (`"key_1"`) when
  deploying.
- **Derivation ID:** The choice of derivation ID is critical. Using the user's
  principal is common for encrypting user-specific data. Ensure you consistently
  use the correct ID for both encryption and decryption key derivation requests.
- **Error Handling:** Robust error handling is essential, especially around
  backend calls and the decryption process, as failures can leave data
  inaccessible.

## Conclusion

Vetkeys provide a native ICP solution for decentralized encryption key
management. While the flow involves a few steps, particularly for decryption, it
eliminates the need for external key servers and ties data access directly to
user identity on the Internet Computer. By setting up the backend endpoints and
leveraging the `ic-vetkd-utils` library on the frontend, you can implement
robust end-to-end encryption for your application's data.

Check out the [Canister Cloud](https://github.com/jakepeg/canister_app/) for the
full codebase context. _(Link added, assuming this is the correct repo)_

## Coming in the Future

- Implementing file sharing with vetkeys (handling decryption for recipients).
- Minimal MVP app demonstrating vetkey functionality.
- Detailed documentation of the vetkey implementation in `Canister Cloud`.

I hope this guide helps you integrate vetkey encryption into your own ICP
projects!
