<script lang="ts">
  import { onMount } from "svelte";
  import FileSelect from "../Upload/FileSelect.svelte";
  import { filesStore } from "$lib/services/files";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/services/auth";
  import Upload from "../Upload/Upload.svelte";

  import {
    createActor,
    CryptoService,
  } from "@shipstone-labs/ic-vetkd-notes-client";
  import { VetKeyService } from "$lib/vetkeys/encrypt";

  let vetKeyService: VetKeyService;
  export let auth: AuthStateAuthenticated;

  onMount(() => {
    auth.filesService.reload();
  });

  function goToDetails(file_id: bigint) {
    goto(`/details?fileId=${file_id}`);
  }
</script>

<div>
  <button>Test Encrypted Upload</button>
</div>

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
      <!-- <Dialog.Root>
        <Dialog.Trigger class={buttonVariants({ variant: "outline" })}
          >Upload</Dialog.Trigger
        >
        <Dialog.Content class="sm:max-w-[425px]"> -->
      {#if $authStore.state === "uninitialized"}
        <h1 class="title-1">Loading...</h1>
      {:else if $authStore.state === "authenticated"}
        <Upload auth={$authStore} />
      {:else}
        Login in dum dum
      {/if}
      <!-- <Upload auth={$authStore} /> -->
      <!-- </Dialog.Content>
      </Dialog.Root> -->
      {#if $filesStore.files.length > 0}
        <!-- <button variant="outline" on:click={() => (isOpenRequestModal = true)}
          >Request</button -->
        >
        <!-- <button
          class="hidden md:inline-block btn btn-accent"
          on:click={() => (isOpenRequestModal = true)}>Request</button
        > -->
      {/if}
    </div>
  </div>
  {#if $filesStore.files.length > 0}
    <div class="hidden md:block w-full rounded-2xl px-2">
      <table class="table-auto w-full border-spacing-y-2 border-separate">
        <thead class="">
          <tr class="body-2 text-text-200 text-left">
            <th class="body-2 pt-4 pb-2 pl-4">Name</th>
            <th class="body-2 pt-6 pb-2">Access</th>
            <th class="body-2 pt-6 pb-2">Uploaded at</th>
            <th></th>
          </tr>
        </thead>
        <tbody class="">
          {#each $filesStore.files as file}
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
              <td class="body-1">{file.uploadedAt}</td>
              <td
                class="pr-4 rounded-tr-xl rounded-br-xl body-1 w-32 text-right h-[52px]"
              >
                <!-- <button
                  on:click|preventDefault|stopPropagation={() =>
                    openShareModal(file.metadata)}
                  class="btn btn-icon"
                >
                  <ShareIcon />
                </button> -->
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <div class="md:hidden flex flex-col gap-2">
      {#each $filesStore.files as file}
        <a
          class="bg-white rounded-xl py-3 px-4 flex flex-col"
          href="/details?fileId={file.file_id}"
        >
          <div class="flex justify-between items-center mb-3">
            <span class="text-text-100 title-2">
              {#if file.name}
                {file.name}
              {:else}
                <span class="opacity-50">Unnamed file</span>
              {/if}
            </span>
            <span>
              <!-- <button
                on:click|preventDefault|stopPropagation={() =>
                  openShareModal(file.metadata)}
                class="btn btn-icon"
              >
                <ShareIcon />
              </button> -->
            </span>
          </div>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <span class="body-1 text-text-200">Access:</span>
              <span class="body-1 text-text-100">{file.access}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="body-1 text-text-200">Uploaded at:</span>
              <span class="body-1 text-text-100">{file.uploadedAtShort}</span>
            </div>
          </div>
        </a>
      {/each}
    </div>
  {:else}
    <div class="pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6">
      <!-- <PlaceholderLogo /> -->
      <h2 class="">
        Even when you have no documents, rest assured, your data is secure.
      </h2>
      <div class="pt-4 pb-8">
        <!-- <button
          class="btn btn-accent md:w-96"
          on:click|preventDefault={() => (isOpenRequestModal = true)}
          >Create new file request</button
        > -->
      </div>
    </div>
  {/if}
{:else}
  <!-- {unreachable($filesStore)} -->
  Womp Womp
{/if}
