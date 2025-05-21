<script lang="ts">
  import RequestModal from "../RequestModal.svelte";
  import UploadModal from "../Upload/UploadModal.svelte";
  import ShareModal from "../ShareModal.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { onMount } from "svelte";
  import { filesStore } from "$lib/services/files";
  import { unreachable } from "$lib/shared/unreachable";
  import { goto } from "$app/navigation";
  import ShareIcon from "../icons/ShareIcon.svelte";
  import PlaceholderLogo from "../icons/PlaceholderLogo.svelte";
  import type { public_item_metadata } from "../../../../../declarations/backend/backend.did";
  import { Button } from "$lib/components/ui/button";
  import FolderIcon from "../icons/FolderIcon.svelte";
  import ChevronRightIcon from "../icons/ChevronRightIcon.svelte";

  import type { UploadedFile } from "$lib/services/files";

  type Props = {
    auth: AuthStateAuthenticated;
  };
  let { auth }: Props = $props();

  // State
  let isOpenRequestModal = $state(false);
  let isOpenShareModal = $state(false);
  let isOpenUploadModal = $state(false);
  let isCreateFolderModalOpen = $state(false);
  let newFolderName = $state("");
  let shareFileData: public_item_metadata | undefined = $state(undefined);
  let currentFolderId: bigint | undefined = $state(undefined);
  let folderPath = $state<{id: bigint | undefined, name: string}[]>([
    { id: undefined, name: "Root" }
  ]);
  let sortField = $state<"name" | "uploadedAt">("uploadedAt");
  let sortDirection = $state<"asc" | "desc">("desc");

  let sortedItems = $state<UploadedFile[]>([]);

  $effect(() => {
    if ($filesStore.state !== "loaded") {
      sortedItems = [];
      return;
    }
    
    const currentItems = $filesStore.files.filter(item => item.parentId === currentFolderId);
    
    sortedItems = [...currentItems].sort((a, b) => {
      if (a.isFolder && !b.isFolder) return -1;
      if (!a.isFolder && b.isFolder) return 1;
      
      if (sortField === "name") {
        const nameA = a.name || "Unnamed";
        const nameB = b.name || "Unnamed";
        return sortDirection === "asc"
          ? nameA.localeCompare(nameB)
          : nameB.localeCompare(nameA);
      } else {
        const dateA = a.metadata.modified_at;
        const dateB = b.metadata.modified_at;
        return sortDirection === "asc"
          ? dateA < dateB ? -1 : dateA > dateB ? 1 : 0
          : dateA > dateB ? -1 : dateA < dateB ? 1 : 0;
      }
    });
  });
  $effect.root(() => {
    if (sortField || sortDirection) {
      // Re-run sorting when sort parameters change
      if ($filesStore.state === "loaded") {
        const currentItems = $filesStore.files.filter(item => item.parentId === currentFolderId);
        sortedItems = [...currentItems].sort((a, b) => {
          if (a.isFolder && !b.isFolder) return -1;
          if (!a.isFolder && b.isFolder) return 1;
          
          if (sortField === "name") {
            const nameA = a.name || "Unnamed";
            const nameB = b.name || "Unnamed";
            return sortDirection === "asc"
              ? nameA.localeCompare(nameB)
              : nameB.localeCompare(nameA);
          } else {
            const dateA = a.metadata.modified_at;
            const dateB = b.metadata.modified_at;
            return sortDirection === "asc"
              ? dateA < dateB ? -1 : dateA > dateB ? 1 : 0
              : dateA > dateB ? -1 : dateA < dateB ? 1 : 0;
          }
        });
      }
    }
  });

  function toggleSort(field: "name" | "uploadedAt") {
    if (sortField === field) {
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      sortField = field;
      sortDirection = field === "name" ? "asc" : "desc";
    }
  }

  function getSortIndicator(field: "name" | "uploadedAt") {
    if (sortField === field) {
      return sortDirection === "asc" ? "↑" : "↓";
    }
    return "";
  }

  onMount(() => {
    auth.filesService.reload();
  });

  function goToDetails(file_id: bigint) {
    goto(`/details?fileId=${file_id}`);
  }

  function openShareModal(file: public_item_metadata) {
    shareFileData = file;
    isOpenShareModal = true;
  }

  function enterFolder(folderId: bigint, folderName: string) {
    currentFolderId = folderId;
    folderPath = [...folderPath, { id: folderId, name: folderName }];
  }

  function navigateToFolder(index: number) {
    if (index >= folderPath.length) return;
    currentFolderId = folderPath[index].id;
    folderPath = folderPath.slice(0, index + 1);
  }

  async function handleCreateFolder() {
    if (!newFolderName) return;

    try {
      await auth.actor.create_folder(
        newFolderName,
        currentFolderId !== undefined ? [currentFolderId] : []
      );
      
      newFolderName = "";
      isCreateFolderModalOpen = false;
      await auth.filesService.reload();
    } catch (error) {
      console.error("Failed to create folder:", error);
    }
  }
</script>

{#if $filesStore.state === "idle" || $filesStore.state === "loading"}
  <h1 class="title-1">Loading...</h1>
{:else if $filesStore.state === "error"}
  <div class="">
    <h1 class="title-1">My Files</h1>
    <p>Error loading files: {$filesStore.error}</p>
  </div>
{:else if $filesStore.state === "loaded"}
  <div class="flex items-center gap-2 mb-4">
    {#each folderPath as folder, index}
      <button 
        class="hover:text-blue-400 transition-colors"
        onclick={() => navigateToFolder(index)}
      >
        {folder.name}
      </button>
      {#if index < folderPath.length - 1}
        <ChevronRightIcon class="w-4 h-4" />
      {/if}
    {/each}
  </div>

  <div class="flex justify-between items-center mb-6">
    <h1 class="title-1">My Files</h1>
    <div class="flex gap-2">
      <Button onclick={() => isCreateFolderModalOpen = true}>New Folder</Button>
      <Button onclick={() => isOpenUploadModal = true}>Upload</Button>
      <Button onclick={() => isOpenRequestModal = true}>Request</Button>
    </div>
  </div>
  
  {#if sortedItems.length > 0}
    <div class="hidden md:block bg-background w-full rounded-2xl px-2">
      <table class="table-auto w-full border-spacing-y-2 border-separate">
        <thead>
          <tr class="text-left">
            <th
              class="body-2 pt-4 pb-2 pl-4 cursor-pointer"
              onclick={() => toggleSort("name")}
            >
              Name {getSortIndicator("name")}
            </th>
            <th class="body-2 pt-6 pb-2">Access</th>
            <th
              class="body-2 pt-6 pb-2 cursor-pointer"
              onclick={() => toggleSort("uploadedAt")}
            >
              Modified {getSortIndicator("uploadedAt")}
            </th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each sortedItems as item (item.file_id)}
            <tr
              class="hover:drop-shadow-xl cursor-pointer"
              onclick={() => item.isFolder ? enterFolder(item.file_id, item.name) : goToDetails(item.file_id)}
            >
              <td class="pl-4 rounded-tl-xl rounded-bl-xl body-1 flex items-center gap-2">
                {#if item.isFolder}
                  <FolderIcon class="w-5 h-5" />
                {:else}
                  <div class="w-5 h-5">📄</div>
                {/if}
                {#if item.name}
                  {item.name}
                {:else}
                  <span class="opacity-50">Unnamed</span>
                {/if}
              </td>
              <td class="body-1">{item.access}</td>
              <td class="body-1">{item.uploadedAtShort}</td>
              <td class="pr-4 rounded-tr-xl rounded-br-xl body-1 w-32 text-right h-[52px]">
                {#if !item.isFolder}
                  <button
                    onclick={(e) => {
                      e.stopPropagation();
                      openShareModal(item.metadata);
                    }}
                    class="btn btn-icon"
                  >
                    <ShareIcon />
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="md:hidden flex flex-col gap-2">
      <div class="flex justify-between items-center mb-2 px-1">
        <span class="body-2">Sort by:</span>
        <div class="flex gap-2">
          <button
            class={`px-2 py-1 rounded ${sortField === "name" ? "bg-primary text-white" : "bg-background"}`}
            onclick={() => toggleSort("name")}
          >
            Name {getSortIndicator("name")}
          </button>
          <button
            class={`px-2 py-1 rounded ${sortField === "uploadedAt" ? "bg-primary text-white" : "bg-background"}`}
            onclick={() => toggleSort("uploadedAt")}
          >
            Date {getSortIndicator("uploadedAt")}
          </button>
        </div>
      </div>

      {#each sortedItems as item}
        <div 
          class="bg-background rounded-xl py-3 px-4 flex flex-col"
          onclick={() => item.isFolder ? enterFolder(item.file_id, item.name) : goToDetails(item.file_id)}
        >
          <div class="flex justify-between items-center mb-3">
            <span class="title-2 flex items-center gap-2">
              {#if item.isFolder}
                <FolderIcon class="w-5 h-5" />
              {:else}
                <div class="w-5 h-5">📄</div>
              {/if}
              {#if item.name}
                {item.name}
              {:else}
                <span class="opacity-50">Unnamed</span>
              {/if}
            </span>
            <span>
              {#if !item.isFolder}
                <button
                  onclick={(e) => {
                    e.stopPropagation();
                    openShareModal(item.metadata);
                  }}
                  class="btn btn-icon"
                >
                  <ShareIcon />
                </button>
              {/if}
            </span>
          </div>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <span class="body-1">Access:</span>
              <span class="body-1">{item.access}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="body-1">Modified:</span>
              <span class="body-1">{item.uploadedAtShort}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6">
      <PlaceholderLogo />
      <h2>
        {#if currentFolderId === undefined}
          No files found. Upload or request documents to get started.
        {:else}
          This folder is empty.
        {/if}
      </h2>
    </div>
  {/if}
{:else}
  {unreachable($filesStore)}
{/if}

{#if isCreateFolderModalOpen}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
    <div class="bg-background p-6 rounded-xl w-96">
      <h2 class="text-xl mb-4">Create New Folder</h2>
      <input
        type="text"
        bind:value={newFolderName}
        placeholder="Folder name"
        class="w-full p-2 mb-4 bg-background-200 rounded"
      />
      <div class="flex justify-end gap-2">
        <Button variant="outline" onclick={() => {
          isCreateFolderModalOpen = false;
          newFolderName = "";
        }}>Cancel</Button>
        <Button onclick={handleCreateFolder}>Create</Button>
      </div>
    </div>
  </div>
{/if}

<RequestModal bind:isOpen={isOpenRequestModal} {auth} />
<UploadModal bind:isOpen={isOpenUploadModal} {auth} {currentFolderId} />
{#if shareFileData}
  <ShareModal
    {auth}
    bind:isOpen={isOpenShareModal}
    bind:fileData={shareFileData}
    on:shared={() => auth.filesService.reload()}
  />
{/if}

<style>
  tbody tr {
    transition: transform 0.2s ease-in-out;
  }

  tbody tr:hover {
    transform: translate(10px, 0);
  }

  .md\:hidden > div {
    transition: transform 0.2s ease-in-out;
  }

  .md\:hidden > div:hover {
    transform: translate(10px, 0);
  }
</style>
