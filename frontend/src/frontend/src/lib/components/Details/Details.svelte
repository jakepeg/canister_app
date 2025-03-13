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

  export let auth: AuthStateAuthenticated;

  const decryptService: DecryptService = new DecryptService(auth);
  const objectUrls = new ObjectUrlManager();

  function getFileId() {
    return parseInt($page.url.searchParams.get("fileId") || "");
  }

  type State =
    | {
        type: "uninitialized";
      }
    | {
        type: "loading";
      }
    | {
        type: "loaded";
        name: string;
        dataType: string;
        uploadDate: string;
        downloadUrl: string;
        isOpenShareModal: boolean;
        originalMetadata: file_metadata;
      }
    | {
        type: "error";
        error: string;
      };

  let state: State = {
    type: "uninitialized",
  };

  let isEditing = false;
  let editedName = "";

  onMount(async () => {
    initialize();
  });

  onDestroy(() => {
    if (decryptService) decryptService.abort();
    objectUrls.clear();
  });

  function openShareDialog() {
    if (state.type === "loaded") {
      state = {
        ...state,
        isOpenShareModal: true,
      };
    }
  }

  async function initialize() {
    decryptService.reset();

    const fileId = BigInt(getFileId());

    try {
      const file = await decryptService.decryptFile({
        fileId,
      });

      if (file === "aborted") {
        console.log("file download/decrypt aborted");

        state = {
          type: "error",
          error: "File not found.",
        };
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

  function saveEdit() {
    console.log("Saving new name:", editedName); // Debugging statement
    if (state.type === "loaded") {
      state.name = editedName;
    }
    isEditing = false; // Finish editing
  }

  function startEdit() {
    if (state.type === "loaded") {
      editedName = state.name; // Set current name as initial value
      isEditing = true; // Start editing mode
    }
  }

  // Type guard to check if state is in the 'loaded' state
  function isLoadedState(
    state: State,
  ): state is Extract<State, { type: "loaded" }> {
    return state.type === "loaded";
  }
</script>

<section style="padding-bottom:5rem">
  <a href="/" class="btn btn-ghost text-sm" style="padding-left:0">
    <BackIcon /> Back to files
  </a>

  {#if state.type === "loading" || state.type === "uninitialized"}
    <div class="title-1 mb-2 mt-3 text-text-200">Loading...</div>
    <DecryptProgress progress={$decryptService} />
  {:else if state.type === "error"}
    <ErrorMessage class="mt-6">{state.error}</ErrorMessage>
  {:else if isLoadedState(state)}
    <div class="flex items-center justify-between mt-3 mb-2">
      {#if isEditing}
        <input
          type="text"
          bind:value={editedName}
          class="title-1"
          on:blur={saveEdit}
          autofocus
        />
        <TickIcon on:click={saveEdit} />
      {:else}
        <h1 id="DocName" class="title-1">
          {#if state.name}
            {state.name}
          {:else}
            <span class="opacity-50">Unnamed file</span>
          {/if}
        </h1>
        <button on:click={startEdit} class="btn btn-ghost">
          <EditIcon />
        </button>
      {/if}
    </div>

    <p class=" text-text-200 text-sm">Uploaded: {state.uploadDate}</p>

    <div class="flex gap-2">
      <a
        href={state.downloadUrl}
        class="btn btn-ghost"
        style="padding-left:0"
        download={state.name}
      >
        <DownloadIcon />
      </a>

      <button class="btn btn-ghost" on:click={openShareDialog}>
        <ShareIcon />
      </button>
    </div>

    <FilePreview
      file={{
        objectUrl: state.downloadUrl,
        dataType: state.dataType,
      }}
    />

    <button class="btn btn-ghost" style="padding-left:0">
      <DeleteIcon />
    </button>

    <ShareModal
      {auth}
      bind:isOpen={state.isOpenShareModal}
      bind:fileData={state.originalMetadata}
    />
  {:else}
    {unreachable(state)}
  {/if}
</section>
