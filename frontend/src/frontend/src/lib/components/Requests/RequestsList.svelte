<script lang="ts">
  import RequestModal from "$lib/components/RequestModal.svelte";
  import UploadModal from "$lib/components/Upload/UploadModal.svelte";
  import PlaceholderLogo from "$lib/components/icons/PlaceholderLogo.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { requestsStore } from "$lib/services/requests";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  export let auth: AuthStateAuthenticated;

  onMount(() => {
    auth.requestService.init();
  });

  let isOpenRequestModal = false;
  let isOpenUploadModal = false;
</script>

<section>
  <RequestModal bind:isOpen={isOpenRequestModal} {auth} />
  <UploadModal bind:isOpen={isOpenUploadModal} {auth} />
  <!-- <RequestModal
    bind:isOpen={isOpenRequestModal}
    on:request-created={() => auth.requestService.reload()}
    {auth}
  /> -->

  {#if $requestsStore.state === "idle" || $requestsStore.state === "loading"}
    <h1 class="title-1">Loading...</h1>
  {:else if $requestsStore.state === "error"}
    <div class="">
      <h1 class="title-1">Requests</h1>
      <p>Error loading requests: {$requestsStore.error}</p>
    </div>
  {:else if $requestsStore.state === "loaded"}
    <div class="flex justify-between items-center mb-6">
      <h1 class="title-1">Requests</h1>
      <div>
        <Button variant="outline" onclick={() => (isOpenUploadModal = true)}
          >Upload</Button
        >
        <Button variant="outline" onclick={() => (isOpenRequestModal = true)}
          >Request</Button
        >
      </div>
    </div>
    {#if $requestsStore.requests.length > 0}
      <div class="hidden md:block bg-background w-full rounded-2xl px-2">
        <table class="table-auto w-full border-spacing-y-2 border-separate">
          <thead class="">
            <tr class="body-2 text-left">
              <th class="body-2 pt-4 pb-2 pl-4">Request</th>

              <th class="body-2 pt-6 pb-2">File</th>
              <th class="body-2 pt-6 pb-2">Created</th>
              <th class="body-2 pt-6 pb-2">Link</th>
            </tr>
          </thead>
          <tbody class="">
            {#each $requestsStore.requests as request}
              <tr>
                <td
                  class="pl-4 bg-background rounded-tl-xl rounded-bl-xl body-1 h-[52px]"
                  >{request.group_name}</td
                >
                <td class="body-1">{request.name}</td>
                <td class="body-1">{request.formattedDateShort}</td>
                <td class="pr-4 rounded-tr-xl rounded-br-xl body-1">
                  <a
                    href="/upload?alias={request.group_alias}"
                    class="underline text-accent-100"
                  >
                    {request.group_alias}
                  </a>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <!-- <div class="md:hidden flex flex-col gap-2">
        {#each $requestsStore.requests as request}
          <div class="bg-white rounded-xl py-3 px-4 flex flex-col">
            <div class="mb-3">
              <span class="text-text-100 title-2">
                {#if request.name}
                  {request.name}
                {:else}
                  <span class="opacity-50">Unnamed request</span>
                {/if}
              </span>
            </div>
            <div class="flex flex-col gap-2">
              <div class="flex justify-between items-center">
                <span class="body-1 text-text-200">Access:</span>
                <span class="body-1 text-text-100">{request.access}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="body-1 text-text-200">Created:</span>
                <span class="body-1 text-text-100"
                  >{request.formattedDateShort}</span
                >
              </div>
              <div class="flex justify-between items-center">
                <span class="body-1 text-text-200">Alias:</span>
                <span class="body-1 text-text-100">
                  <a
                    href="/upload?alias={request.alias}"
                    class="underline text-accent-100"
                  >
                    {request.alias}
                  </a>
                </span>
              </div>
            </div>
          </div>
        {/each}
      </div> -->
    {:else}
      <div class="pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6">
        <PlaceholderLogo />
        <h2 class="">You don't have any upload requests.</h2>
      </div>
    {/if}
  {/if}
</section>
