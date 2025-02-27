<script lang="ts">
  import { onMount } from "svelte";
  import * as vetkd from "ic-vetkd-utils";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { toBytes } from "viem";
  import pLimit from "p-limit";
  import { enumIs } from "$lib/shared/enums";
  import type { get_alias_info_response } from "../../../../../declarations/backend/backend.did";

  export let auth: AuthStateAuthenticated;
  let file: File | null = null;
  let decryptedContent: Uint8Array | null = null;
  let error: string | null = null;

  const CHUNK_SIZE = 2_000_000;
  const aborted = false;
  const uploadType: UploadType = {
    type: "self",
    fileName: "",
  };
  const onStarted: (totalSizeBytes: number) => void = () => {};
  const onAborted: () => void = () => {};
  const onCompleted: (file_id: bigint) => void = () => {};
  const onError: (message: string) => void = () => {};
  const onChunkUploaded: (chunkId: number, size: number) => void = () => {};
  const dataType: string = "";

  type UploadType =
    | {
        type: "request";
        fileInfo: Extract<get_alias_info_response, { Ok: any }>["Ok"];
      }
    | {
        type: "self";
        fileName: string;
      };

  onMount(async () => await refreshNotes());

  async function refreshNotes() {
    try {
      // TODO
      return;
    } catch (err) {
      error = "Failed to load files";
    }
  }

  async function handleUpload() {
    try {
      if (!file) {
        throw new Error("No file selected");
      }
      // TODO
      // 1. Encrypt file

      // Part 1
      // Generate a random seed
      const seed = window.crypto.getRandomValues(new Uint8Array(32));
      console.log("seed", seed);
      // Get the user_id (e.g. principal)
      const user_id = auth.authClient.getIdentity().getPrincipal().toString();
      console.log("user_id", user_id);

      // Part 2 - Transform the file into a format that can be encrypted
      // Transform the file into an array buffer
      const fileBuffer = await file.arrayBuffer();
      console.log("fileBuffer", fileBuffer);
      // Transform the array buffer into an encoded message (Uint8Array)
      const encodedMessage = new Uint8Array(fileBuffer);
      console.log("encodedMessage", encodedMessage);

      // Part 3 - Public key
      // We are getting the public key from the backend
      const publicKeyResponse = await auth.actor?.vetkd_public_key();
      if (!publicKeyResponse) {
        console.error("Error getting public key, empty response");
        return;
      }
      if ("Err" in publicKeyResponse) {
        console.error("Error getting public key", publicKeyResponse.Err);
        return;
      }
      const publicKey = publicKeyResponse.Ok as Uint8Array;
      console.log("publicKey", publicKey);

      // Part 4 - Encrypt the file
      const encryptedFile = vetkd.IBECiphertext.encrypt(
        publicKey,
        toBytes(user_id!),
        encodedMessage,
        seed, // Check if this makes sense
      );
      console.log("encryptedFile", encryptedFile);

      const encryptedData = encryptedFile.serialize();
      console.log("encryptedData", encryptedData);

      // 2. Upload encrypted file
      // Split file into chunks of 2MB.
      const numChunks = Math.ceil(encryptedData.length / CHUNK_SIZE);
      console.log("numChunks: ", numChunks);

      try {
        onStarted(encryptedData.length);
        console.log("onStarted done");

        const firstChunk = encryptedData.subarray(0, CHUNK_SIZE);
        console.log("firstChunk done");
        let fileId: bigint = 0n;
        if (uploadType.type === "request") {
          fileId = uploadType.fileInfo.file_id;
          console.log("fileId for request: ", fileId);
          const res = await auth.actor.upload_file({
            file_id: fileId,
            file_content: firstChunk,
            owner_key: publicKey, // Make sure this is correct
            file_type: dataType,
            num_chunks: BigInt(numChunks),
          });
          console.log("res done for request");

          if (enumIs(res, "Err")) {
            onError(
              "An error occurred while uploading the file. Please try again.",
            );
            return;
          }
        } else {
          fileId = await auth.actor.upload_file_atomic({
            content: firstChunk,
            owner_key: publicKey, // Make sure this is correct
            name: file.name,
            file_type: dataType,
            num_chunks: BigInt(numChunks),
          });
          // console.log("fileId for self: ", fileId);
        }

        onChunkUploaded(0, firstChunk.length);

        if (aborted) {
          onAborted();
          return;
        }

        await uploadChunks(encryptedData, fileId, onChunkUploaded);

        if (aborted) {
          onAborted();
          return;
        }

        onCompleted(fileId);
      } catch (err) {
        console.error(err);
        onError(
          "An error occurred while uploading the file. Please try again.",
        );
      }

      return;
    } catch (err) {
      error = "Upload failed: " + err;
    }
  }

  async function uploadChunks(
    content: Uint8Array,
    fileId: bigint,
    onChunkUploaded: (chunkId: number, size: number) => void,
  ) {
    const numChunks = Math.ceil(content.length / CHUNK_SIZE);

    // Create upload pool, supporting upto 5 parallel uploads.
    const uploadPool = pLimit(5);

    // Prepare upload requests.
    const uploadRequests = Array.from(
      { length: numChunks - 1 },
      (_, i) => i + 1,
    ).map((i) =>
      uploadPool(async () => {
        if (aborted) {
          return;
        }
        const chunk = content.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
        await auth.actor.upload_file_continue({
          file_id: fileId,
          contents: chunk,
          chunk_id: BigInt(i),
        });
        onChunkUploaded(i, chunk.length);
      }),
    );

    await Promise.all(uploadRequests);
  }

  async function handleDecrypt(fileId: bigint) {
    try {
      // TODO
      // 1. Decrypt file
      // 2. Understand where to get derived_public_key_bytes (is it the public_key method?)
      // 3. Understand where to get derivation_id (it was suggested to use the caller)

      // Part 1
      // Gernearte a random seed
      const seed = window.crypto.getRandomValues(new Uint8Array(32));
      // Initialize the trasnport secret key
      const transportSecretKey = new vetkd.TransportSecretKey(seed);

      // Get the user_id (e.g. principal)
      const user_id = auth.authClient.getIdentity().getPrincipal().toString();

      // Part 2 - Public key
      // We are getting the public key from the backend
      const publicKeyResponse = await auth.actor.vetkd_public_key();
      if (!publicKeyResponse) {
        console.error("Error getting public key, empty response");
        return;
      }
      if ("Err" in publicKeyResponse) {
        console.error("Error getting public key", publicKeyResponse.Err);
        return;
      }
      const publicKey = publicKeyResponse.Ok as Uint8Array;

      // Part3 - Encrypted key
      // We are getting the encrypted key from the backend by passing the public key
      const privateKeyResponse = await auth.actor?.vetkd_encrypted_key(
        transportSecretKey.public_key(),
      );
      if (!privateKeyResponse) {
        console.error("Error getting encrypted key, empty response");
        return;
      }
      if ("Err" in privateKeyResponse) {
        console.error("Error getting encrypted key", privateKeyResponse.Err);
        return;
      }
      // We extract it from the an object {key, string} and type cast it to Uint8Array
      const encryptedKey = privateKeyResponse.Ok as Uint8Array;

      // Part 4 - Getting the file
      const encryptedFile = await auth.actor.download_file(fileId, 0n); // Download a file with the specific fileId and 0n means its one chunk

      // Part 5 - Decrypting the file
      try {
        const key = transportSecretKey.decrypt(
          encryptedKey,
          publicKey!,
          toBytes(user_id!),
        );
        const ibeCiphertext = vetkd.IBECiphertext.deserialize(
          encryptedFile.data as Uint8Array,
        );
        const decryptedData = ibeCiphertext.decrypt(key);
        return { decryptedData, ...encryptedFile };
      } catch (e) {
        console.error("Error decrypting transfer", e);
      }

      return;
    } catch (err) {
      error = "Decryption failed: " + err;
    }
  }

  function handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (files && files.length > 0) {
      file = files[0];
    }
  }
</script>

<div class="container">
  <div class="upload-section">
    <input type="file" on:change={handleFileChange} />
    <button on:click={handleUpload}>Encrypt & Upload</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="file-list">
    <h2>Encrypted Files</h2>
    <ul>
      <!-- {#each notes as note (note.id)}
        <li>
          {note.encrypted_text.substring(0, 20)}...
          <button on:click={() => handleDecrypt(note.id)}>Decrypt</button>
        </li>
      {/each} -->
    </ul>
  </div>

  {#if decryptedContent}
    <div class="preview">
      <h3>Decrypted Content</h3>
      <pre>{new TextDecoder().decode(decryptedContent)}</pre>
    </div>
  {/if}
</div>

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  .upload-section {
    margin-bottom: 2rem;
    display: flex;
    gap: 1rem;
  }

  .file-list ul {
    list-style: none;
    padding: 0;
  }

  /* .file-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    border-bottom: 1px solid #eee;
  } */

  .error {
    color: red;
    margin: 1rem 0;
  }

  .preview pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 4px;
  }
</style>
