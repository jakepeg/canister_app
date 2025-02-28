<script lang="ts">
  import { onMount } from "svelte";
  import * as vetkd from "ic-vetkd-utils";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { toBytes } from "viem";
  import pLimit from "p-limit";
  import { enumIs } from "$lib/shared/enums";
  import { flatten } from "$lib/shared/flatten";
  import { formatUploadDate, formatUploadDateShort } from "$lib/shared/dates";
  import { filesStore } from "$lib/services/files";
  import { unreachable } from "$lib/shared/unreachable";
  import type {
    file_metadata,
    get_alias_info_response,
  } from "../../../../../declarations/backend/backend.did";

  export let auth: AuthStateAuthenticated;
  let file: File | null = null;
  let decryptedContent: Uint8Array | null = null;
  let error: string | null = null;
  let selectedFileId: bigint | null = null;
  let decryptedFileName: string = "";
  let isDecrypting = false;
  let decryptProgress = {
    step: "initializing",
    totalChunks: 0,
    currentChunk: 0,
  };

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

  type UploadedFile = {
    name: string;
    access: string;
    uploadedAt: string;
    uploadedAtShort: string;
    file_id: bigint;
    metadata: file_metadata;
  };

  onMount(async () => {
    await refreshFiles();
  });

  async function refreshFiles() {
    try {
      await auth.filesService.reload();
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
      const user_id = auth.authClient.getIdentity().getPrincipal();
      console.log("user_id: ", user_id);
      const user_id_bytes = user_id.toUint8Array();
      console.log("user_id: ", user_id.toString());
      console.log("user_id_bytes: ", user_id_bytes);

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
        user_id_bytes,
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

  async function handleDecrypt(fileId: bigint, fileName: string) {
    try {
      isDecrypting = true;
      selectedFileId = fileId;
      decryptedFileName = fileName || "Unknown file";

      // Part 4 - Getting the file
      const files = await loadFiles();
      filesStore.setLoaded(files, false);
      console.log("Files: ", files);

      // Start downloading
      decryptProgress = {
        ...decryptProgress,
        step: "downloading",
      };

      // Download the first chunk
      let downloadedFile = await auth.actor.download_file(fileId, 0n);
      console.log("downloadedFile: ", downloadedFile);

      if (enumIs(downloadedFile, "found_file")) {
        const totalChunks = Number(downloadedFile.found_file.num_chunks);
        decryptProgress = {
          ...decryptProgress,
          totalChunks,
          currentChunk: 1,
        };

        // Download additional chunks if needed
        for (let i = 1; i < downloadedFile.found_file.num_chunks; i++) {
          const downloadedChunk = await auth.actor.download_file(
            fileId,
            BigInt(i),
          );

          if (enumIs(downloadedChunk, "found_file")) {
            decryptProgress = {
              ...decryptProgress,
              currentChunk: i + 1,
            };

            const chunk = downloadedChunk.found_file.contents;

            // Merge chunks
            const mergedArray = new Uint8Array(
              downloadedFile.found_file.contents.length + chunk.length,
            );
            mergedArray.set(downloadedFile.found_file.contents, 0);
            mergedArray.set(chunk, downloadedFile.found_file.contents.length);

            downloadedFile.found_file.contents = mergedArray;
          } else if (enumIs(downloadedChunk, "not_found_file")) {
            throw new Error("Chunk not found");
          } else if (enumIs(downloadedChunk, "permission_error")) {
            throw new Error(
              "Permission error: You don't have access to this file",
            );
          }
        }

        // Start decryption
        decryptProgress = {
          ...decryptProgress,
          step: "decrypting",
        };

        try {
          // Generate a random seed
          const seed = window.crypto.getRandomValues(new Uint8Array(32));
          // Initialize the transport secret key
          const transportSecretKey = new vetkd.TransportSecretKey(seed);
          console.log("transportSecretKey: ", transportSecretKey);

          // Get the user_id (principal)
          const user_id = auth.authClient.getIdentity().getPrincipal();
          console.log("user_id: ", user_id);
          const user_id_bytes = user_id.toUint8Array();
          console.log("user_id: ", user_id.toString());
          console.log("user_id_bytes: ", user_id_bytes);

          // Get public key from the backend
          const publicKeyResponse = await auth.actor.vetkd_public_key();
          if (!publicKeyResponse) {
            throw new Error("Error getting public key: empty response");
          }
          if ("Err" in publicKeyResponse) {
            throw new Error(
              `Error getting public key: ${publicKeyResponse.Err}`,
            );
          }
          const publicKey = publicKeyResponse.Ok as Uint8Array;
          console.log("publicKey: ", publicKey);

          // Get encrypted key from the backend
          const privateKeyResponse = await auth.actor?.vetkd_encrypted_key(
            transportSecretKey.public_key(),
          );
          if (!privateKeyResponse) {
            throw new Error("Error getting encrypted key: empty response");
          }
          if ("Err" in privateKeyResponse) {
            throw new Error(
              `Error getting encrypted key: ${privateKeyResponse.Err}`,
            );
          }
          const encryptedKey = privateKeyResponse.Ok as Uint8Array;
          console.log("encryptedKey: ", encryptedKey);

          // Decrypt the file
          const key = transportSecretKey.decrypt(
            encryptedKey,
            publicKey,
            user_id_bytes, // Tis could be the issue
          );
          console.log("key: ", key);

          const ibeCiphertext = vetkd.IBECiphertext.deserialize(
            downloadedFile.found_file.contents as Uint8Array,
          );
          console.log("ibeCiphertext: ", ibeCiphertext);

          const decryptedData = ibeCiphertext.decrypt(key);
          console.log("decryptedData: ", decryptedData);
          decryptedContent = decryptedData;
          console.log("decryptedContent: ", decryptedContent);

          console.log("File decrypted successfully");
        } catch (e) {
          console.error("Decryption error:", e);
          throw new Error(
            "Failed to decrypt file. You may be able to access this file with a different browser, as the decryption key is stored in the browser.",
          );
        }
      } else if (enumIs(downloadedFile, "not_found_file")) {
        throw new Error("File not found");
      } else if (enumIs(downloadedFile, "permission_error")) {
        throw new Error("Permission error: You don't have access to this file");
      } else if (enumIs(downloadedFile, "not_uploaded_file")) {
        throw new Error("File not uploaded");
      } else {
        unreachable(downloadedFile);
      }

      return;
    } catch (err) {
      error = "Decryption failed: " + err;
    }
  }

  async function loadFiles(): Promise<UploadedFile[]> {
    const files = flatten(
      await Promise.all([
        auth.actor.get_shared_files(),
        auth.actor.get_requests(),
      ]),
    );

    const uploadedFiles: UploadedFile[] = [];

    for (const file of files) {
      if (enumIs(file.file_status, "uploaded")) {
        // Determine the sharing status
        let nShared = file.shared_with ? file.shared_with.length : 0;
        let accessMessage = "";
        switch (nShared) {
          case 0:
            accessMessage = "Only You";
            break;
          case 1:
            accessMessage = "You & 1 other";
            break;
          default:
            accessMessage = "You & " + nShared + " others";
        }

        uploadedFiles.push({
          name: file.file_name,
          access: accessMessage,
          uploadedAt: formatUploadDate(file.file_status.uploaded.uploaded_at),
          uploadedAtShort: formatUploadDateShort(
            file.file_status.uploaded.uploaded_at,
          ),
          file_id: file.file_id,
          metadata: file,
        });
      }
    }
    console.log("uploadedFiles: ", uploadedFiles);

    return uploadedFiles;
  }

  function handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (files && files.length > 0) {
      file = files[0];
    }
  }

  function downloadDecryptedFile() {
    if (!decryptedContent) return;

    const blob = new Blob([decryptedContent]);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = decryptedFileName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
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

  <div class="file-list mb-8">
    <h2 class="title-2 mb-4">Your Encrypted Files</h2>

    {#if $filesStore.state === "idle" || $filesStore.state === "loading"}
      <p class="text-center py-4">Loading files...</p>
    {:else if $filesStore.state === "error"}
      <p class="text-center py-4 text-red-500">
        Error loading files: {$filesStore.error}
      </p>
    {:else if $filesStore.state === "loaded"}
      {#if $filesStore.files.length > 0}
        <div class="overflow-x-auto">
          <table class="w-full table-auto border-separate border-spacing-y-2">
            <thead>
              <tr class="text-left text-gray-500">
                <th class="px-4 py-2">Name</th>
                <th class="px-4 py-2">Access</th>
                <th class="px-4 py-2">Uploaded at</th>
                <th class="px-4 py-2">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each $filesStore.files as file}
                <tr class="rounded-xl ition-colors">
                  <td class="px-4 py-3 rounded-l-xl">
                    {#if file.name}
                      {file.name}
                    {:else}
                      <span class="opacity-50">Unnamed file</span>
                    {/if}
                  </td>
                  <td class="px-4 py-3">{file.access}</td>
                  <td class="px-4 py-3">{file.uploadedAt}</td>
                  <td class="px-4 py-3 rounded-r-xl space-x-2">
                    <button
                      on:click={() => handleDecrypt(file.file_id, file.name)}
                      class="px-3 py-1 bg-green-500 text-white text-sm rounded hover:bg-green-600 transition-colors"
                      disabled={isDecrypting && selectedFileId === file.file_id}
                    >
                      {#if isDecrypting && selectedFileId === file.file_id}
                        Decrypting...
                      {:else}
                        Decrypt
                      {/if}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <p class="text-center py-4 bg-white rounded-xl">
          You don't have any encrypted files yet.
        </p>
      {/if}
    {/if}
  </div>

  {#if decryptedContent}
    <div class="preview bg-white rounded-xl p-4 shadow">
      <div class="flex justify-between items-center mb-4">
        <h3 class="title-3">Decrypted Content: {decryptedFileName}</h3>
        <button
          on:click={downloadDecryptedFile}
          class="px-3 py-1 bg-blue-500 text-white text-sm rounded hover:bg-blue-600 transition-colors"
        >
          Download File
        </button>
      </div>

      <div class="preview-content max-h-[400px] overflow-auto">
        {#if decryptedContent.length > 100000}
          <p class="mb-2 text-gray-500">
            File is too large to preview. Please download it.
          </p>
          <p class="text-sm">Size: {decryptedContent.length} bytes</p>
        {:else}
          <pre
            class="whitespace-pre-wrap word-wrap break-all bg-gray-50 p-4 rounded">{new TextDecoder()
              .decode(decryptedContent)
              .substring(0, 10000)}{decryptedContent.length > 10000
              ? "...(truncated)"
              : ""}</pre>
        {/if}
      </div>
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

  /* .file-list ul {
    list-style: none;
    padding: 0;
  } */

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
