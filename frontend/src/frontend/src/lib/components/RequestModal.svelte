<script lang="ts">
  import { page } from "$app/stores";
  import { createEventDispatcher } from "svelte";
  import Modal from "./Modal.svelte";
  import CopyIcon from "./icons/CopyIcon.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";

  export let isOpen = false;
  export let auth: AuthStateAuthenticated;
  export let savedTemplates: string[] = []; // Mocked saved templates list

  let requestLink: URL | null = null;
  let loading: boolean = false;
  let requestName: string = "";
  let copied = false;
  let documents: string[] = [""]; // At least one document field by default
  let selectedTemplate: string = "";
  let saveAsTemplate: boolean = false;

  const dispatch = createEventDispatcher<{
    "request-created": void;
    "request-completed": void;
  }>();

  function addDocument() {
    documents = [...documents, ""];
  }

  function removeDocument(index: number) {
    documents = documents.filter((_, i) => i !== index);
  }

  function loadTemplate(template: string) {
    if (template) {
      requestName = template;
      documents = ["Document 1", "Document 2"]; // Mocked template documents
    }
  }

  async function updateRequestUrl(e) {
    loading = true;
    const formData = new FormData(e.target);
    const data: any = {};
    for (let field of formData) {
      const [key, value] = field;
      data[key] = value;
    }

    if (data.requestName && !data.requestLink) {
      requestName = data.requestName;
      const alias = await auth.actor.request_file(data.requestName);
      requestLink = new URL($page.url.origin + "/upload");
      requestLink.searchParams.append("alias", alias);
    }
    loading = false;

    dispatch("request-created");
  }

  function close() {
    if (requestLink) {
      dispatch("request-completed");
    }
    isOpen = false;
    requestName = "";
    requestLink = null;
    documents = [""];
  }

  async function copyText() {
    if (requestLink) {
      await navigator.clipboard.writeText(requestLink.toString());
      copied = true;
    }
  }
</script>

<div>
  <Modal bind:isOpen title="Create Request" on:cancelled={close}>
    <form class="w-full md:w-96" on:submit|preventDefault={updateRequestUrl}>
      <div>
        <label for="template" class="input-label">Load Template</label>
        <select
          id="template"
          bind:value={selectedTemplate}
          on:change={() => loadTemplate(selectedTemplate)}
          class="input"
        >
          <option value="">Select a template</option>
          {#each savedTemplates as template}
            <option value={template}>{template}</option>
          {/each}
        </select>
      </div>
      <div class="mt-3">
        <label for="requestName" class="input-label">Request Name</label>
        <input
          type="text"
          required
          class="input"
          id="requestName"
          placeholder="Request name"
          bind:value={requestName}
          disabled={!!requestLink}
          readonly={!!requestLink}
        />
      </div>
      <div class="mt-3">
        <label class="input-label">Documents</label>
        {#each documents as doc, index}
          <div class="flex gap-2 items-center mt-2">
            <input
              type="text"
              class="input flex-1"
              placeholder="Document name"
              bind:value={documents[index]}
            />
            {#if documents.length > 1}
              <button
                type="button"
                class="btn btn-danger"
                on:click={() => removeDocument(index)}>✖</button
              >
            {/if}
          </div>
        {/each}
        <button
          type="button"
          class="btn btn-secondary mt-2"
          on:click={addDocument}>+ Add Document</button
        >
      </div>
      <div class="mt-3 flex items-center gap-2">
        <input
          type="checkbox"
          id="saveTemplate"
          bind:checked={saveAsTemplate}
        />
        <label for="saveTemplate">Save as Template</label>
      </div>
      <div class="mt-10">
        {#if loading}
          <button type="submit" class="btn btn-accent btn-full" disabled
            >Generating link...</button
          >
        {:else if requestLink}
          <button type="button" class="btn btn-accent btn-full" on:click={close}
            >Request sent, close this window</button
          >
        {:else}
          <button type="submit" class="btn btn-accent btn-full"
            >Generate link</button
          >
        {/if}
      </div>
    </form>
  </Modal>
</div>
