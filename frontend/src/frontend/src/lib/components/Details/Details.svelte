<script lang="ts">
  import { page } from "$app/stores";
  import FilePreview from "$lib/components/FilePreview.svelte";
  import ShareModal from "$lib/components/ShareModal.svelte";
  import BackIcon from "$lib/components/icons/BackIcon.svelte";
  import DownloadIcon from "$lib/components/icons/DownloadIcon.svelte";
  import ShareIcon from "$lib/components/icons/ShareIcon.svelte";
  import DeleteIcon from "$lib/components/icons/DeleteIcon.svelte";
  import EditIcon from "$lib/components/icons/EditIcon.svelte";
  import TickIcon from "$lib/components/icons/TickIcon.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { DecryptService } from "$lib/services/decrypt";
  import { ObjectUrlManager } from "$lib/services/objectUrls";
  import { unreachable } from "$lib/shared/unreachable";
  import { onDestroy, onMount } from "svelte";
  import type { file_metadata } from "../../../../../declarations/backend/backend.did";
  import ErrorMessage from "../ErrorMessage.svelte";
  import DecryptProgress from "./DecryptProgress.svelte";
  import { goto } from "$app/navigation";
  import { enumIs } from "$lib/shared/enums";

  export let auth: AuthStateAuthenticated;

  const decryptService: DecryptService = new DecryptService(auth);
  const objectUrls = new ObjectUrlManager();

  function getFileId() {
    return parseInt($page.url.searchParams.get("fileId") || "");
  }

  type State =
    | { type: "uninitialized" }
    | { type: "loading" }
    | {
        type: "loaded";
        name: string;
        dataType: string;
        uploadDate: string;
        downloadUrl: string;
        isOpenShareModal: boolean;
        originalMetadata: file_metadata;
      }
    | { type: "error"; error: string };

  let state: State = { type: "uninitialized" };
  let showDeletePrompt = false;
  let isEditing = false;
  let editedName = "";

  onMount(async () => {
    initialize();
  });

  onDestroy(() => {
    if (decryptService) decryptService.abort();
    objectUrls.clear();
  });

  async function initialize() {
    decryptService.reset();
    const fileId = BigInt(getFileId());

    try {
      const file = await decryptService.decryptFile({ fileId });
      if (file === "aborted") {
        state = { type: "error", error: "File not found." };
        return;
      }
      state = {
        type: "loaded",
        downloadUrl: objectUrls.createObjectURLFromArrayBuffer(
          file.contents,
          file.dataType,
        ),
        dataType: file.dataType,
        name: file.name,
        uploadDate: file.uploadDate,
        originalMetadata: file.originalMetadata,
        isOpenShareModal: false,
      };
    } catch (e: unknown) {
      state = {
        type: "error",
        error:
          e instanceof Error
            ? e.message || "Error decrypting file."
            : "Error opening file: " + e,
      };
    }
  }

  async function DeleteFile() {
    try {
      const fileId = BigInt(getFileId());
      const response = await auth.actor.delete_file(fileId);

      if (enumIs(response, "ok")) {
        await goto("/");
        console.log("File deleted with fileId:", fileId);
      } else {
        console.error("Delete failed:", response);
      }
    } catch (error) {
      console.error("Error deleting file:", error);
    }
    showDeletePrompt = false;
  }

  async function saveEdit() {
    if (state.type === "loaded") {
      try {
        const fileId = BigInt(getFileId());
        const response = await auth.actor.rename_file(fileId, editedName);

        console.log("fileId:", fileId);
        console.log("newName:", editedName);

        if (enumIs(response, "ok")) {
          state.name = editedName;
          isEditing = false;
        } else {
          console.error("Rename failed:", response);
        }
      } catch (error) {
        console.error("Error renaming file:", error);
      }
    }
  }

  function startEdit() {
    if (state.type === "loaded") {
      editedName = state.name;
      isEditing = true;
    }
  }
</script>

<section style="padding-bottom:5rem">
  <a href="/" class="btn btn-ghost text-sm" style="padding-left:0; height:0">
    <BackIcon /> Back to files
  </a>

  {#if state.type === "loading" || state.type === "uninitialized"}
    <div class="title-1 mb-2 mt-3 text-text-200">Loading...</div>
    <DecryptProgress progress={$decryptService} />
  {:else if state.type === "error"}
    <ErrorMessage class="mt-6">{state.error}</ErrorMessage>
  {:else}
    <div class="flex items-center gap-10 mt-3 mb-2">
      {#if isEditing}
        <input
          type="text"
          bind:value={editedName}
          class="title-1"
          on:blur={saveEdit}
        />
        <TickIcon on:click={saveEdit} />
      {:else}
        <h1 id="DocName" class="title-1">{state.name || "Unnamed file"}</h1>
        <button on:click={startEdit} class="btn btn-ghost">
          <EditIcon />
        </button>
      {/if}
    </div>
    <p class="text-text-200 text-sm">Uploaded: {state.uploadDate}</p>
    <div class="flex gap-2">
      <a
        href={state.downloadUrl}
        class="btn btn-ghost"
        style="padding-left:0"
        download={state.name}
      >
        <DownloadIcon />
      </a>
      <button
        class="btn btn-ghost"
        on:click={() => {
          if (state.type === "loaded") {
            state.isOpenShareModal = true;
          }
        }}
      >
        <ShareIcon />
      </button>
    </div>
    <FilePreview
      file={{ objectUrl: state.downloadUrl, dataType: state.dataType }}
    />
    <div class="flex items-center gap-10 mt-3">
      {#if showDeletePrompt}
        <div class="flex items-center gap-2 border p-2 rounded">
          <span>Delete?</span>
          <button on:click={DeleteFile} class="cursor-pointer">
            <TickIcon />
          </button>

          <button
            on:click={() => (showDeletePrompt = false)}
            class="cursor-pointer"
            style="color:#C73B3B">X</button
          >
        </div>
      {:else}
        <button
          class="btn btn-ghost"
          style="padding-left:0"
          on:click={() => (showDeletePrompt = true)}
        >
          <DeleteIcon />
        </button>
      {/if}
    </div>
    <ShareModal
      {auth}
      bind:isOpen={state.isOpenShareModal}
      bind:fileData={state.originalMetadata}
    />
  {/if}
</section>
