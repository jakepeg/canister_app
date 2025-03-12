<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { encrypt } from "$lib/crypto/upload";
  import type { AuthState } from "$lib/services/auth";
  import { unreachable } from "$lib/shared/unreachable";
  import LoadingIndicator from "../LoadingIndicator.svelte";
  import UploadIcon from "../icons/UploadIcon.svelte";
  import SuccessIcon from "../icons/SuccessIcon.svelte";
  import { VetkdCryptoService } from "$lib/vetkeys/vetkdCrypto";

  export let auth: AuthState;

  let alias = $page.url.searchParams.get("alias") || "";
  let groupInfo = null;
  let error: string | null = null;
  let loading = true;
  let fileUploads = [];
  let allUploaded = false;

  onMount(async () => {
    if (!alias) {
      error = "No alias provided";
      loading = false;
      return;
    }

    try {
      // Get the group info using the provided alias
      const result = await auth.actor.get_group_by_alias(alias);

      if ("Err" in result) {
        error = "File request not found";
        loading = false;
        return;
      }

      groupInfo = result.Ok;

      // Initialize the file uploads array
      fileUploads = groupInfo.files.map((file) => ({
        fileId: file.file_id,
        fileName: file.file_name,
        alias: file.alias,
        file: null,
        status: "pending", // pending, uploading, uploaded, error
        progress: 0,
        error: null,
      }));

      loading = false;
    } catch (e) {
      console.error(e);
      error = "Failed to load file information";
      loading = false;
    }
  });

  function handleFileSelect(fileIndex, event) {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    fileUploads[fileIndex].file = files[0];
    fileUploads[fileIndex].status = "ready";
    fileUploads = [...fileUploads]; // Trigger reactivity
  }

  async function uploadFile(fileIndex) {
    const fileUpload = fileUploads[fileIndex];
    if (
      !fileUpload.file ||
      fileUpload.status === "uploading" ||
      fileUpload.status === "uploaded"
    ) {
      return;
    }

    fileUpload.status = "uploading";
    fileUpload.progress = 0;
    fileUploads = [...fileUploads];

    try {
      const fileReader = new FileReader();
      const filePromise = new Promise((resolve, reject) => {
        fileReader.onload = () => resolve(fileReader.result);
        fileReader.onerror = reject;
      });

      fileReader.readAsArrayBuffer(fileUpload.file);
      const arrayBuffer = await filePromise;

      // Encrypt the file
      const { encryptedContent, fileKey } = await encrypt(
        new Uint8Array(arrayBuffer),
        auth.keys.publicKey,
      );

      const encoded = Array.from(encryptedContent);

      // Upload the file
      await auth.actor.upload_file({
        file_id: fileUpload.fileId,
        file_content: encoded,
        file_type: fileUpload.file.type,
        owner_key: Array.from(fileKey),
        num_chunks: 1,
      });

      fileUpload.status = "uploaded";
      fileUpload.progress = 100;
      fileUploads = [...fileUploads];

      // Check if all files are uploaded
      checkAllUploaded();
    } catch (e) {
      console.error(e);
      fileUpload.status = "error";
      fileUpload.error = "Upload failed";
      fileUploads = [...fileUploads];
    }
  }

  function checkAllUploaded() {
    allUploaded = fileUploads.every((file) => file.status === "uploaded");
  }

  async function uploadAll() {
    for (let i = 0; i < fileUploads.length; i++) {
      if (fileUploads[i].status === "ready") {
        await uploadFile(i);
      }
    }
  }

  function goHome() {
    goto("/");
  }
</script>

<div class="container mx-auto p-4">
  {#if loading}
    <div class="flex justify-center items-center h-64">
      <LoadingIndicator />
    </div>
  {:else if error}
    <div class="p-4 bg-red-100 text-red-700 rounded-lg mb-4">
      {error}
    </div>
    <button class="btn btn-accent" on:click={goHome}>Go Home</button>
  {:else if allUploaded}
    <div class="bg-white rounded-lg p-6 shadow-md mb-4">
      <div class="flex flex-col items-center justify-center text-center p-8">
        <div class="mb-4 text-green-500">
          <SuccessIcon size={48} />
        </div>
        <h2 class="title-1 mb-2">All files uploaded successfully!</h2>
        <p class="mb-6">All documents have been securely uploaded</p>
        <button class="btn btn-accent" on:click={goHome}>Return Home</button>
      </div>
    </div>
  {:else}
    <div class="bg-white rounded-lg p-6 shadow-md mb-4">
      <h1 class="title-1 mb-4">Multiple Document Upload</h1>
      <p class="mb-4">
        <strong>{groupInfo.requester.username}</strong> has requested the following
        documents:
      </p>

      <div class="space-y-4 mb-6">
        {#each fileUploads as fileUpload, index}
          <div class="border rounded-lg p-4">
            <h3 class="title-2 mb-2">{fileUpload.fileName}</h3>

            {#if fileUpload.status === "uploaded"}
              <div class="flex items-center text-green-500">
                <SuccessIcon class="mr-2" />
                <span>Uploaded successfully</span>
              </div>
            {:else if fileUpload.status === "uploading"}
              <div class="w-full bg-gray-200 rounded-full h-2.5 mb-2">
                <div
                  class="bg-blue-600 h-2.5 rounded-full"
                  style="width: {fileUpload.progress}%"
                ></div>
              </div>
              <p>Uploading... {fileUpload.progress}%</p>
            {:else if fileUpload.status === "error"}
              <div class="text-red-500 mb-2">{fileUpload.error}</div>
              <div class="flex gap-2">
                <input
                  type="file"
                  id="file-{index}"
                  class="hidden"
                  on:change={(e) => handleFileSelect(index, e)}
                />
                <label for="file-{index}" class="btn btn-secondary">
                  <UploadIcon class="mr-2" />
                  Select File
                </label>
                <button
                  class="btn btn-primary"
                  disabled={!fileUpload.file}
                  on:click={() => uploadFile(index)}
                >
                  Retry Upload
                </button>
              </div>
            {:else}
              <div class="flex gap-2">
                <input
                  type="file"
                  id="file-{index}"
                  class="hidden"
                  on:change={(e) => handleFileSelect(index, e)}
                />
                <label for="file-{index}" class="btn btn-secondary">
                  <UploadIcon class="mr-2" />
                  {fileUpload.file ? fileUpload.file.name : "Select File"}
                </label>
                <button
                  class="btn btn-primary"
                  disabled={!fileUpload.file}
                  on:click={() => uploadFile(index)}
                >
                  Upload
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <div class="flex justify-between">
        <span class="text-sm text-gray-500">
          Request group: {groupInfo.group_name}
        </span>
        <button
          class="btn btn-accent"
          on:click={uploadAll}
          disabled={!fileUploads.some((f) => f.status === "ready")}
        >
          Upload All
        </button>
      </div>
    </div>
  {/if}
</div>
