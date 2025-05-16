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

  type Props = {
    auth: AuthStateAuthenticated;
  };
  let { auth }: Props = $props();

  // State (Using Runes where appropriate within the component)
  let isOpenRequestModal = $state(false);
  let isOpenShareModal = $state(false);
  let isOpenUploadModal = $state(false);
  let shareFileData: public_item_metadata | undefined = $state(undefined);

  // Sorting variables - Use $state for reactivity
  let sortField = $state<"name" | "uploadedAt">("uploadedAt");
  let sortDirection = $state<"asc" | "desc">("desc");

  // Computed sorted files - Use $derived
  let sortedFiles = $derived(() => {
    if ($filesStore.state !== "loaded") {
      return [];
    }
    // Get dependencies for derived state
    const field = sortField;
    const direction = sortDirection;
    const files = $filesStore.files;

    // Perform sorting
    return [...files].sort((a, b) => {
      if (field === "name") {
        const nameA = a.name || "Unnamed file";
        const nameB = b.name || "Unnamed file";
        return direction === "asc"
          ? nameA.localeCompare(nameB)
          : nameB.localeCompare(nameA);
      } else {
        // uploadedAt
        let dateA = 0n;
        let dateB = 0n;
        // Safely access nested properties
        if (a.metadata?.file_status && "uploaded" in a.metadata.file_status) {
          dateA = a.metadata.file_status.uploaded.uploaded_at;
        }
        if (b.metadata?.file_status && "uploaded" in b.metadata.file_status) {
          dateB = b.metadata.file_status.uploaded.uploaded_at;
        }
        // Comparison logic remains the same
        return direction === "asc"
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

  // Toggle sort function
  function toggleSort(field: "name" | "uploadedAt") {
    console.log($filesStore);
    if (sortField === field) {
      // Toggle direction if clicking the same field
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      // Set new field and default direction
      sortField = field;
      sortDirection = field === "name" ? "asc" : "desc"; // Default: A-Z for name, newest first for date
    }
  }

  // Get sort indicator
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
</script>

{#if $filesStore.state === "idle" || $filesStore.state === "loading"}
  <h1 class="title-1">Loading...</h1>
{:else if $filesStore.state === "error"}
  <div class="">
    <h1 class="title-1">My Files</h1>
    <p>Error loading files: {$filesStore.error}</p>
  </div>
{:else if $filesStore.state === "loaded"}
  <div class="flex justify-between items-center mb-6">
    <h1 class="title-1">My Files</h1>
    <div>
      <Button variant="outline" onclick={() => (isOpenUploadModal = true)}
        >Upload</Button
      >
      <Button variant="outline" onclick={() => (isOpenRequestModal = true)}
        >Request</Button
      >
    </div>
  </div>
  {#if $filesStore.files.length > 0}
    <div class="hidden md:block w-full rounded-2xl px-2">
      <table class="table-auto w-full border-spacing-y-2 border-separate">
        <thead class="">
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
              Uploaded {getSortIndicator("uploadedAt")}
            </th>
            <th></th>
          </tr>
        </thead>
        <tbody class="">
          {#each sortedFiles() as file (file.file_id)}
            <tr
              class="hover:drop-shadow-xl cursor-pointer"
              onclick={() => goToDetails(file.file_id)}
            >
              <td class="pl-4 rounded-tl-xl rounded-bl-xl body-1">
                {#if file.name}
                  {file.name}
                {:else}
                  <span class="opacity-50">Unnamed file</span>
                {/if}
              </td>
              <td class=" body-1">{file.access}</td>
              <td class="body-1">{file.uploadedAtShort}</td>
              <td
                class="pr-4 rounded-tr-xl rounded-br-xl body-1 w-32 text-right h-[52px]"
              >
                <button
                  onclick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    openShareModal(file.metadata);
                  }}
                  class="btn btn-icon"
                >
                  <ShareIcon />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <div class="md:hidden flex flex-col gap-2">
      <!-- Mobile sorting controls -->
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

      {#each sortedFiles() as file}
        <a
          class="bg-background rounded-xl py-3 px-4 flex flex-col"
          href="/details?fileId={file.file_id}"
        >
          <div class="flex justify-between items-center mb-3">
            <span class="title-2">
              {#if file.name}
                {file.name}
              {:else}
                <span class="opacity-50">Unnamed file</span>
              {/if}
            </span>
            <span>
              <button
                onclick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  openShareModal(file.metadata);
                }}
                class="btn btn-icon"
              >
                <ShareIcon />
              </button>
            </span>
          </div>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <span class="body-1">Access:</span>
              <span class="body-1">{file.access}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="body-1">Uploaded:</span>
              <span class="body-1">{file.uploadedAtShort}</span>
            </div>
          </div>
        </a>
      {/each}
    </div>
  {:else}
    <div class="pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6">
      <PlaceholderLogo />
      <h2 class="">
        No files found. Upload or request documents to get started.
      </h2>
    </div>
  {/if}
{:else}
  {unreachable($filesStore)}
{/if}
<div class="md:hidden fixed bottom-0 left-0 right-0 bg-background-200 p-4">
  <button
    class="btn btn-accent btn-full"
    onclick={() => (isOpenRequestModal = true)}>Create new file request</button
  >
</div>
<RequestModal bind:isOpen={isOpenRequestModal} {auth} />
<UploadModal bind:isOpen={isOpenUploadModal} {auth} />
{#if shareFileData}
  <ShareModal
    {auth}
    bind:isOpen={isOpenShareModal}
    bind:fileData={shareFileData}
    on:shared={() => auth.filesService.reload()}
  />
{/if}

<style lang="postcss">
  tbody tr {
    transition: transform 0.2s ease-in-out;
  }

  tbody tr:hover {
    transform: translateX(10px);
  }

  .md\:hidden > a {
    transition: transform 0.2s ease-in-out;
  }

  .md\:hidden > a:hover {
    transform: translateX(10px);
  }
</style>
