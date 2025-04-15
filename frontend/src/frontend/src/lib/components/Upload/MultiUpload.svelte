<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import LinkedInIcon from "../icons/linkedin.svelte";
  import XIcon from "../icons/x-logo.svelte";
  // import { encrypt } from "$lib/crypto/upload";
  import {
    UploadService,
    uploadInProgress,
    type UploadType,
  } from "$lib/services/upload";
  import { enumIs } from "$lib/shared/enums";
  import UploadIcon from "../icons/UploadIcon.svelte";
  import type {
    AuthStateAuthenticated,
    AuthStateUnauthenticated,
  } from "$lib/services/auth";
  import { authService, authStore } from "$lib/services/auth";
  import type { group_info } from "../../../../../declarations/backend/backend.did";

  export let auth: AuthStateAuthenticated | AuthStateUnauthenticated;
  let uploadService: UploadService | null = null;

  let alias = $page.url.searchParams.get("alias") || "";
  let groupInfo: group_info | null = null;
  let error: string | null = null;
  let loading = true;
  let fatalError = false;

  type FileUpload = {
    fileId: bigint;
    fileName: string;
    file: File | null;
    status: "pending" | "ready" | "uploading" | "uploaded" | "error";
    progress: number;
    error: string | null;
  };

  let fileUploads: FileUpload[] = [];
  let allUploaded = false;

  onMount(async () => {
    if (!alias) {
      error = "No alias provided";
      loading = false;
      return;
    }

    console.log("alias: ", alias);

    try {
      const result = await auth.actor.get_group_by_alias(alias);

      if (enumIs(result, "Ok")) {
        groupInfo = result.Ok;
        fileUploads = groupInfo.files.map((file) => ({
          fileId: BigInt(file.file_id),
          fileName: file.file_name,
          file: null,
          status: "pending",
          progress: 0,
          error: null,
        }));
        loading = false;
      } else if (enumIs(result, "Err")) {
        fatalError = true;
        error = "Request not found or already uploaded";
        loading = false;
        console.log("error: ", result);
      }
    } catch (e) {
      console.error(e);
      error = "Failed to load file information";
      loading = false;
    }
  });

  function handleFileSelect(fileIndex: number, event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (!files || files.length === 0) return;

    fileUploads[fileIndex].file = files[0];
    fileUploads[fileIndex].status = "ready";
    fileUploads = [...fileUploads];
  }

  async function uploadFile(fileIndex: number) {
    const fileUpload = fileUploads[fileIndex];
    if (!fileUpload.file || fileUpload.status === "uploading") return;

    try {
      fileUploads[fileIndex].status = "uploading";
      fileUploads[fileIndex].progress = 0;
      fileUploads = [...fileUploads];

      const uploadService = new UploadService(auth);

      const uploadType: UploadType = {
        type: "request",
        fileInfo: {
          file_id: fileUpload.fileId,
          file_name: fileUpload.fileName,
          user: groupInfo!.requester,
        },
      };

      await uploadService.uploadFile({
        file: fileUpload.file,
        dataType: fileUpload.file.type,
        uploadType,
        onAborted: () => {
          fileUploads[fileIndex].status = "ready";
          fileUploads = [...fileUploads];
        },
        onError: (msg) => {
          fileUploads[fileIndex].status = "error";
          fileUploads[fileIndex].error = msg;
          fileUploads = [...fileUploads];
        },
        onCompleted: () => {
          fileUploads[fileIndex].status = "uploaded";
          fileUploads[fileIndex].progress = 100;
          fileUploads = [...fileUploads];
          checkAllUploaded();
        },
        onChunkUploaded: (chunkId, chunkSize) => {
          // Update progress based on your chunking strategy
          const progress = Math.min(
            100,
            (chunkId + 1) * ((100 / fileUpload.file!.size) * chunkSize),
          );
          fileUploads[fileIndex].progress = progress;
          fileUploads = [...fileUploads];
        },
        onStarted: (totalBytes) => {
          // Initialize progress
          fileUploads[fileIndex].progress = 0;
          fileUploads = [...fileUploads];
        },
      });
    } catch (e) {
      console.error(e);
      fileUploads[fileIndex].status = "error";
      fileUploads[fileIndex].error = "Upload failed";
      fileUploads = [...fileUploads];
    }
  }

  function checkAllUploaded() {
    allUploaded = fileUploads.every((f) => f.status === "uploaded");
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
  <!-- If not logged in condition -->
  {#if $authStore.state !== "authenticated"}
    <div class="flex flex-col items-center text-center mt-16 mb-4">
      <h1 class="text-4xl font-bold">Store, Share & Collect Documents</h1>
      <h2 class="text-lg mt-2 text-gray-400">
        Sovereign data management designed for privacy, secured on the
        blockchain.
      </h2>
    </div>
  {/if}
  <!-- End if not logged in condition -->
  {#if loading}
    <div class="flex justify-center items-center h-64">
      <!-- <LoadingIndicator /> -->
      LoadingIndicator goes here
    </div>
  {:else if error}
    <div class="p-4 bg-red-100 text-red-700 rounded-lg mb-4">
      {error}
    </div>
    <button class="btn btn-accent" on:click={goHome}>Go Home</button>
  {:else if allUploaded}
    <div class="bg-background rounded-lg p-6 mb-4">
      <div class="flex flex-col items-center justify-center text-center p-8">
        <h2 class="title-1 mb-2">All documents have been securely uploaded</h2>
        <!-- <button class="btn btn-accent" on:click={goHome}>Return Home</button> -->
      </div>
    </div>
  {:else}
    <div class="bg-background rounded-lg p-6 mb-4">
      <h1 class="title-1 mb-4">{groupInfo?.group_name} Document Upload</h1>
      <p class="mb-4">
        <strong>{groupInfo?.requester.username}</strong> has requested the following
        documents:
      </p>

      <div class="space-y-4 mb-6">
        {#each fileUploads as fileUpload, index}
          <div class="border blue-border p-4">
            <h3 class="title-2 mb-2">{fileUpload.fileName}</h3>

            {#if fileUpload.status === "uploaded"}
              <div class="flex items-center text-green-500">
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
                <label
                  for="file-{index}"
                  class="btn btn-secondary cursor-pointer"
                >
                  <UploadIcon />
                  Select File
                </label>
                <button
                  class="btn btn-accent"
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
                <label
                  for="file-{index}"
                  class="btn btn-secondary cursor-pointer"
                >
                  <UploadIcon />
                  {fileUpload.file ? fileUpload.file.name : "Select File"}
                </label>
                <button
                  class="btn btn-accent"
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

      <!-- <div class="flex justify-between">
        <button
          class="btn btn-accent"
          on:click={uploadAll}
          disabled={!fileUploads.some((f) => f.status === "ready")}
        >
          Upload All
        </button>
      </div> -->
    </div>
  {/if}
</div>
<!-- If not logged in condition -->
{#if $authStore.state !== "authenticated"}
  <div class="text-center text-gray-400">
    <h2 class="text-lg">
      With Canister, your file transfers are encrypted to be secure and private.
    </h2>
    <a href="/" class="btn btn-cta mt-6"> Learn More </a>
  </div>

  <!-- Fixed Social Icons Section -->
  <div class="fixed-social-icons">
    <a href="https://www.linkedin.com/company/canister-cloud/" target="_blank">
      <span class="h-6 w-6">
        <LinkedInIcon />
      </span>
    </a>
    <a href="https://x.com/CanisterCloud" target="_blank">
      <span class="h-6 w-6">
        <XIcon />
      </span>
    </a>
  </div>
{/if}

<!-- End if not logged in condition -->
<style>
  .fixed-social-icons {
    position: fixed;
    bottom: 15px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 10px;
  }
</style>
