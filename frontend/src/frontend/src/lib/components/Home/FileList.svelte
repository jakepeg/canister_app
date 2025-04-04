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
  import type { file_metadata } from "../../../../../declarations/backend/backend.did";
  import { Button } from "$lib/components/ui/button";

  export let auth: AuthStateAuthenticated;
  let isOpenRequestModal = false;
  let isOpenShareModal = false;
  let isOpenUploadModal = false;
  let shareFileData: file_metadata | undefined = undefined;

  // Sorting variables
  let sortField: "name" | "uploadedAt" = "uploadedAt";
  let sortDirection: "asc" | "desc" = "desc"; // Default newest first

  // Computed sorted files
  $: sortedFiles =
    $filesStore.state === "loaded"
      ? [...$filesStore.files].sort((a, b) => {
          if (sortField === "name") {
            const nameA = a.name || "Unnamed file";
            const nameB = b.name || "Unnamed file";
            return sortDirection === "asc"
              ? nameA.localeCompare(nameB)
              : nameB.localeCompare(nameA);
          } else {
            const dateA = a.uploadedAt;
            const dateB = b.uploadedAt;

            if (sortDirection === "asc") {
              // Ascending: Oldest first
              if (dateA < dateB) return -1;
              if (dateA > dateB) return 1;
              return 0;
            } else {
              // Descending: Newest first
              if (dateB < dateA) return -1; // If B is older than A, A comes first
              if (dateB > dateA) return 1; // If B is newer than A, A comes after
              return 0;
            }
          }
        })
      : [];

  // Toggle sort function
  function toggleSort(field: "name" | "uploadedAt") {
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

  function openShareModal(file: file_metadata) {
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
              on:click={() => toggleSort("name")}
            >
              Name {getSortIndicator("name")}
            </th>
            <th class="body-2 pt-6 pb-2">Access</th>
            <th
              class="body-2 pt-6 pb-2 cursor-pointer"
              on:click={() => toggleSort("uploadedAt")}
            >
              Uploaded {getSortIndicator("uploadedAt")}
            </th>
            <th></th>
          </tr>
        </thead>
        <tbody class="">
          {#each sortedFiles as file}
            <tr
              class="hover:drop-shadow-xl cursor-pointer"
              on:click={() => goToDetails(file.file_id)}
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
                  on:click|preventDefault|stopPropagation={() =>
                    openShareModal(file.metadata)}
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
            on:click={() => toggleSort("name")}
          >
            Name {getSortIndicator("name")}
          </button>
          <button
            class={`px-2 py-1 rounded ${sortField === "uploadedAt" ? "bg-primary text-white" : "bg-background"}`}
            on:click={() => toggleSort("uploadedAt")}
          >
            Date {getSortIndicator("uploadedAt")}
          </button>
        </div>
      </div>

      {#each sortedFiles as file}
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
                on:click|preventDefault|stopPropagation={() =>
                  openShareModal(file.metadata)}
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
    on:click={() => (isOpenRequestModal = true)}>Create new file request</button
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
