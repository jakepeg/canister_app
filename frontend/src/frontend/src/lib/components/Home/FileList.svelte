<script lang="ts">
  import RequestModal from "../RequestModal.svelte";
  import UploadModal from "../Upload/UploadModal.svelte";
  import ShareModal from "../ShareModal.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { onMount, onDestroy } from "svelte";
  import { filesStore } from "$lib/services/files";
  import { unreachable } from "$lib/shared/unreachable";
  import { goto } from "$app/navigation";
  import ShareIcon from "../icons/ShareIcon.svelte";
  import PlaceholderLogo from "../icons/PlaceholderLogo.svelte";
  import type {
    public_item_metadata,
    item_id,
  } from "../../../../../declarations/backend/backend.did";
  import { Button } from "$lib/components/ui/button";
  import FolderIcon from "../icons/FolderIcon.svelte";
  import ChevronRightIcon from "../icons/ChevronRightIcon.svelte";
  import { EllipsisIcon, EllipsisVertical } from "lucide-svelte";
  import { page } from "$app/stores";

  import type { UploadedFile } from "$lib/services/files";
  import { clickOutside } from "$lib/utils/clickOutside"; // Helper for dropdown

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
  let currentFolderId: item_id | undefined = $state(undefined);
  let folderPath = $state<{ id: item_id | undefined; name: string }[]>([
    { id: undefined, name: "Root" },
  ]);
  let sortField = $state<"name" | "uploadedAt">("uploadedAt");
  let sortDirection = $state<"asc" | "desc">("desc");

  let sortedItems = $state<UploadedFile[]>([]);

  // More options dropdown state
  let activeDropdownItemId: item_id | null = $state(null);

  // Rename modal state
  let isRenameModalOpen = $state(false);
  let itemToRename: public_item_metadata | undefined = $state(undefined);
  let renameNewName = $state("");
  let renameError = $state<string | null>(null);

  // Delete state (can be simpler if just using window.confirm)
  let deleteError = $state<string | null>(null);

  $effect(() => {
    if ($filesStore.state !== "loaded") {
      sortedItems = [];
      return;
    }
    sortedItems = [...$filesStore.files].sort((a, b) => {
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
          ? dateA < dateB
            ? -1
            : dateA > dateB
              ? 1
              : 0
          : dateA > dateB
            ? -1
            : dateA < dateB
              ? 1
              : 0;
      }
    });
  });

  $effect(() => {
    if (auth && auth.filesService) {
      auth.filesService.loadFolderContents(currentFolderId);
    }
  });

  function getCanisterId() {
    const canisterId = parseInt($page.url.searchParams.get("canisterId") || "");
    return canisterId.toString();
  }

  onMount(() => {
    if (auth && auth.filesService) {
      if ($filesStore.state === "idle") {
        auth.filesService.loadFolderContents(undefined);
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

  function goToDetails(canisterIdStr: string, file_id: bigint) {
    goto(`/canister/${canisterIdStr}/details?fileId=${file_id}`);
  }

  function openShareModalForItem(itemMeta: public_item_metadata) {
    shareFileData = itemMeta;
    isOpenShareModal = true;
    activeDropdownItemId = null; // Close dropdown
  }

  function enterFolder(folderId: item_id, folderName: string) {
    currentFolderId = folderId;
    folderPath = [...folderPath, { id: folderId, name: folderName }];
  }

  function navigateToFolder(index: number) {
    if (index >= folderPath.length) return;
    currentFolderId = folderPath[index].id;
    folderPath = folderPath.slice(0, index + 1);
  }

  async function handleCreateFolder() {
    if (!newFolderName.trim()) {
      console.warn("Folder name cannot be empty.");
      // TODO: Show user feedback
      return;
    }
    try {
      const parentFolderOpt: [] | [item_id] =
        currentFolderId !== undefined ? [currentFolderId] : [];
      const result = await auth.actor.create_folder(
        newFolderName,
        parentFolderOpt,
      );
      if ("Err" in result) {
        alert(`Error creating folder: ${result.Err}`);
        return;
      }
      newFolderName = "";
      isCreateFolderModalOpen = false;
      if (auth && auth.filesService) {
        auth.filesService.loadFolderContents(currentFolderId);
      }
    } catch (error) {
      console.error("Failed to create folder:", error);
      alert(`Failed to create folder: ${error}`);
    }
  }

  function toggleItemDropdown(itemId: item_id) {
    if (activeDropdownItemId === itemId) {
      activeDropdownItemId = null;
    } else {
      activeDropdownItemId = itemId;
    }
  }

  function closeAllDropdowns() {
    activeDropdownItemId = null;
  }

  function openRenameModalForItem(itemMeta: public_item_metadata) {
    itemToRename = itemMeta;
    renameNewName = itemMeta.name;
    isRenameModalOpen = true;
    renameError = null;
    activeDropdownItemId = null; // Close dropdown
  }

  async function handleRenameItem() {
    if (!itemToRename || !renameNewName.trim()) {
      renameError = "New name cannot be empty.";
      return;
    }
    renameError = null;
    try {
      const result = await auth.actor.rename_item(
        itemToRename.id,
        renameNewName,
      );
      if ("Err" in result) {
        renameError = result.Err;
        // alert(`Error renaming item: ${result.Err}`); // Or use renameError in modal
      } else {
        isRenameModalOpen = false;
        itemToRename = undefined;
        renameNewName = "";
        if (auth && auth.filesService) {
          auth.filesService.loadFolderContents(currentFolderId);
        }
      }
    } catch (error) {
      console.error("Failed to rename item:", error);
      renameError = `Failed to rename item: ${error}`;
      // alert(`Failed to rename item: ${error}`);
    }
  }

  async function handleDeleteItem(itemMeta: public_item_metadata) {
    activeDropdownItemId = null; // Close dropdown
    const itemTypeDisplay = "Folder" in itemMeta.item_type ? "folder" : "file";
    if (
      !window.confirm(
        `Are you sure you want to delete this ${itemTypeDisplay} "${itemMeta.name}"? This action cannot be undone.`,
      )
    ) {
      return;
    }
    deleteError = null;
    try {
      const result = await auth.actor.delete_item(itemMeta.id);
      if ("Err" in result) {
        deleteError = result.Err;
        alert(`Error deleting item: ${result.Err}`); // Show error to user
      } else {
        if (auth && auth.filesService) {
          auth.filesService.loadFolderContents(currentFolderId);
        }
      }
    } catch (error) {
      console.error("Failed to delete item:", error);
      deleteError = `Failed to delete item: ${error}`;
      alert(`Failed to delete item: ${error}`);
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
      <Button onclick={() => (isCreateFolderModalOpen = true)}
        >New Folder</Button
      >
      <Button onclick={() => (isOpenUploadModal = true)}>Upload</Button>
      <Button onclick={() => (isOpenRequestModal = true)}>Request</Button>
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
            <th class="w-12"></th>
            <!-- Column for ... button -->
          </tr>
        </thead>
        <tbody>
          {#each sortedItems as item (item.file_id)}
            <tr class="hover:drop-shadow-xl relative">
              <!-- Added relative for dropdown positioning -->
              <td
                class="pl-4 rounded-tl-xl rounded-bl-xl body-1 flex items-center gap-2 cursor-pointer"
                onclick={() =>
                  item.isFolder
                    ? enterFolder(item.file_id, item.name)
                    : goToDetails(getCanisterId(), item.file_id)}
              >
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
              <td
                class="body-1 cursor-pointer"
                onclick={() =>
                  item.isFolder
                    ? enterFolder(item.file_id, item.name)
                    : goToDetails(getCanisterId(), item.file_id)}
                >{item.access}</td
              >
              <td
                class="body-1 cursor-pointer"
                onclick={() =>
                  item.isFolder
                    ? enterFolder(item.file_id, item.name)
                    : goToDetails(getCanisterId(), item.file_id)}
                >{item.uploadedAtShort}</td
              >
              <td
                class="pr-4 rounded-tr-xl rounded-br-xl body-1 text-right h-[52px]"
              >
                <button
                  class="p-2 hover:bg-gray-200 rounded-full"
                  aria-label="More options for {item.name}"
                  onclick={(e) => {
                    e.stopPropagation();
                    toggleItemDropdown(item.file_id);
                  }}
                >
                  <EllipsisVertical class="w-5 h-5" />
                </button>
                {#if activeDropdownItemId === item.file_id}
                  <div
                    use:clickOutside={closeAllDropdowns}
                    class="absolute right-0 mt-2 w-48 bg-white border border-gray-200 rounded-md shadow-lg z-10"
                  >
                    <ul>
                      {#if !item.isFolder}
                        <li>
                          <button
                            class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                            onclick={(e) => {
                              e.stopPropagation();
                              openShareModalForItem(item.metadata);
                            }}
                          >
                            Share
                          </button>
                        </li>
                      {/if}
                      <li>
                        <button
                          class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                          onclick={(e) => {
                            e.stopPropagation();
                            openRenameModalForItem(item.metadata);
                          }}
                        >
                          Rename
                        </button>
                      </li>
                      <li>
                        <button
                          class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50"
                          onclick={(e) => {
                            e.stopPropagation();
                            handleDeleteItem(item.metadata);
                          }}
                        >
                          Delete
                        </button>
                      </li>
                    </ul>
                  </div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Mobile View -->
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
        <div class="bg-background rounded-xl py-3 px-4 flex flex-col relative">
          <div
            role="button"
            tabindex="0"
            class="flex-grow"
            onclick={() =>
              item.isFolder
                ? enterFolder(item.file_id, item.name)
                : goToDetails(getCanisterId(), item.file_id)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                item.isFolder
                  ? enterFolder(item.file_id, item.name)
                  : goToDetails(getCanisterId(), item.file_id);
              }
            }}
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
              <!-- Mobile More Options Button -->
              <button
                class="p-1 -mr-1 hover:bg-gray-200 rounded-full z-20"
                aria-label="More options for {item.name}"
                onclick={(e) => {
                  e.stopPropagation();
                  toggleItemDropdown(item.file_id);
                }}
              >
                <EllipsisIcon class="w-5 h-5" />
              </button>
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
          <!-- Mobile Dropdown -->
          {#if activeDropdownItemId === item.file_id}
            <div
              use:clickOutside={closeAllDropdowns}
              class="absolute right-4 top-12 mt-2 w-48 bg-white border border-gray-200 rounded-md shadow-lg z-30"
            >
              <ul>
                {#if !item.isFolder}
                  <li>
                    <button
                      class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                      onclick={(e) => {
                        e.stopPropagation();
                        openShareModalForItem(item.metadata);
                      }}
                    >
                      Share
                    </button>
                  </li>
                {/if}
                <li>
                  <button
                    class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                    onclick={(e) => {
                      e.stopPropagation();
                      openRenameModalForItem(item.metadata);
                    }}
                  >
                    Rename
                  </button>
                </li>
                <li>
                  <button
                    class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50"
                    onclick={(e) => {
                      e.stopPropagation();
                      handleDeleteItem(item.metadata);
                    }}
                  >
                    Delete
                  </button>
                </li>
              </ul>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {:else if $filesStore.state === "loaded"}
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
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40"
  >
    <div class="bg-background p-6 rounded-xl w-96">
      <h2 class="text-xl mb-4">Create New Folder</h2>
      <input
        type="text"
        bind:value={newFolderName}
        placeholder="Folder name"
        class="w-full p-2 mb-4 bg-input border border-border rounded focus:ring-ring focus:border-ring"
        onkeydown={(e) => {
          if (e.key === "Enter") handleCreateFolder();
        }}
      />
      <div class="flex justify-end gap-2">
        <Button
          variant="outline"
          onclick={() => {
            isCreateFolderModalOpen = false;
            newFolderName = "";
          }}>Cancel</Button
        >
        <Button onclick={handleCreateFolder}>Create</Button>
      </div>
    </div>
  </div>
{/if}

{#if isRenameModalOpen && itemToRename}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40"
  >
    <div class="bg-background p-6 rounded-xl w-96">
      <h2 class="text-xl mb-4">
        Rename {"Folder" in itemToRename.item_type ? "Folder" : "File"}
      </h2>
      <input
        type="text"
        bind:value={renameNewName}
        placeholder="New name"
        class="w-full p-2 mb-1 bg-input border border-border rounded focus:ring-ring focus:border-ring"
        onkeydown={(e) => {
          if (e.key === "Enter") handleRenameItem();
        }}
      />
      {#if renameError}
        <p class="text-red-500 text-sm mb-3">{renameError}</p>
      {:else}
        <div class="mb-4"></div>
        <!-- Spacer to keep layout consistent -->
      {/if}
      <div class="flex justify-end gap-2">
        <Button
          variant="outline"
          onclick={() => {
            isRenameModalOpen = false;
            itemToRename = undefined;
            renameNewName = "";
            renameError = null;
          }}>Cancel</Button
        >
        <Button onclick={handleRenameItem}>Save</Button>
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
    on:shared={() => auth.filesService.reload(currentFolderId)}
  />
{/if}

<style>
  tbody tr {
    transition: transform 0.2s ease-in-out;
  }
  /*tbody tr:hover {
    transform: translate(10px, 0);
  } Keep this commented if dropdowns are used */

  .md\:hidden > div.bg-background {
    /* Target the card specifically */
    transition: transform 0.2s ease-in-out;
  }
  /* .md\:hidden > div.bg-background:hover {
    transform: translate(10px, 0);
  } Keep this commented if dropdowns are used */
</style>
