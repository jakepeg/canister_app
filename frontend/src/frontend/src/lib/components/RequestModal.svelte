<script lang="ts">
  import { page } from "$app/stores";
  import { createEventDispatcher } from "svelte";
  import Modal from "./Modal.svelte";
  import CopyIcon from "./icons/CopyIcon.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { enumIs } from "$lib/shared/enums";
  import type { template } from "../../../../declarations/backend/backend.did";

  export let isOpen = false;
  export let auth: AuthStateAuthenticated;

  let requestLink: URL | null = null;
  let loading: boolean = false;
  let requestName: string = "";
  let copied = false;
  let documents: string[] = [""]; // At least one document field by default

  let generatedLinks: string[] = [];
  let groupId: bigint | null = null;

  let selectedTemplate: string = ""; // Change this to just a string
  let saveAsTemplate: boolean = false;
  export let savedTemplates: template[] = [];

  // Fetch user's templates when modal opens
  $: if (isOpen) {
    loadTemplates();
  }

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

  async function loadTemplates() {
    try {
      savedTemplates = await auth.actor.get_user_templates();
      console.log("savedTemplates: ", savedTemplates);
    } catch (error) {
      console.error("Error loading templates:", error);
    }
  }

  async function loadTemplate(templateName: string) {
    if (templateName) {
      try {
        const templateResult = await auth.actor.get_template(templateName);
        console.log("templateResult: ", templateResult);
        if (
          enumIs(templateResult, "some") &&
          templateResult &&
          templateResult.length > 0
        ) {
          const template = templateResult.some;
          if (template) {
            // Add this check
            requestName = template.name || "";
            documents = [...template.file_names];
          }
        } else {
          console.log("Template not found");
          // Optional: Clear fields if template not found
          requestName = "";
          documents = [""];
        }
      } catch (error) {
        console.error("Error loading template:", error);
      }
    }
  }

  async function updateRequestUrl(e) {
    loading = true;
    const validDocuments = documents.filter((doc) => doc.trim() !== "");

    if (validDocuments.length === 0) {
      alert("Please add at least one document name");
      loading = false;
      return;
    }

    if (requestName && validDocuments.length > 0) {
      try {
        const response = await auth.actor.multi_request({
          group_name: requestName,
          file_names: validDocuments,
          save_as_template: saveAsTemplate,
        });

        // Create URL with group alias
        const groupUrl = new URL($page.url.origin + "/upload");
        groupUrl.searchParams.append("alias", response.group_alias);
        generatedLinks = [groupUrl.toString()];
        requestLink = groupUrl;

        // Redirect directly to multi-upload
        // goto(`/upload?alias=${response.group_alias}`);
      } catch (error) {
        console.error("Error creating request:", error);
        alert("Failed to create document request. Please try again.");
      }
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

  // async function copyText(link: string | null = null) {
  //   if (link) {
  //     await navigator.clipboard.writeText(link);
  //   } else if (generatedLinks.length > 0) {
  //     const linksList = generatedLinks
  //       .map((link, i) => `${documents[i] || `Document ${i + 1}`}: ${link}`)
  //       .join("\n");
  //     await navigator.clipboard.writeText(linksList);
  //   } else if (requestLink) {
  //     await navigator.clipboard.writeText(requestLink.toString());
  //   }
  //   copied = true;
  //   setTimeout(() => {
  //     copied = false;
  //   }, 2000);
  // }

  async function copyText(link: string | null = null) {
    if (link) {
      await navigator.clipboard.writeText(link);
    } else if (generatedLinks.length > 0) {
      const linksList = generatedLinks.join("\n");
      await navigator.clipboard.writeText(linksList);
    } else if (requestLink) {
      await navigator.clipboard.writeText(requestLink.toString());
    }

    copied = true;
    setTimeout(() => {
      copied = false;
    }, 2000);
  }
</script>

<div>
  <Modal
    bind:isOpen
    title="Create Request"
    minWidth="min-w-[300px]"
    on:cancelled={close}
  >
    <div class="flex flex-col max-h-[55vh]">
      <div class="overflow-y-auto flex-1 p-2">
        <form
          class="w-full md:w-96"
          on:submit|preventDefault={updateRequestUrl}
        >
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
                <option value={template.name}>{template.name}</option>
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
            <span class="input-label">Documents</span>
            {#each documents as doc, index}
              <div class="flex gap-2 items-center mt-2">
                <input
                  type="text"
                  class="input flex-1"
                  placeholder="Document name"
                  bind:value={documents[index]}
                  disabled={!!requestLink}
                />
                {#if documents.length > 1}
                  <button
                    type="button"
                    class="btn btn-danger"
                    on:click={() => removeDocument(index)}
                    disabled={!!requestLink}>✖</button
                  >
                {/if}
              </div>
            {/each}
            <button
              type="button"
              class="btn btn-secondary mt-2"
              style="font-size:14px"
              on:click={addDocument}>+ Add Document</button
            >
          </div>
          <div class="mt-3 flex items-center gap-2">
            <input
              type="checkbox"
              id="saveTemplate"
              bind:checked={saveAsTemplate}
              disabled={!!requestLink}
            />
            <label for="saveTemplate" style="font-size:14px"
              >Save as Template</label
            >
          </div>

          {#if generatedLinks.length > 0}
            <div class="mt-4 border p-3 rounded-md bg-gray-50">
              <h3 class="font-medium mb-2">Generated Links:</h3>
              <ul class="space-y-2">
                {#each generatedLinks as link, i}
                  <li class="text-sm">
                    <div class="flex justify-between items-center">
                      <span class="font-medium"
                        >{documents[i] || `Document ${i + 1}`}</span
                      >
                      <button
                        type="button"
                        class="text-blue-600 hover:text-blue-800"
                        on:click={() => copyText(link)}
                      >
                        <CopyIcon />
                      </button>
                    </div>
                    <div class="truncate text-gray-500 text-xs">
                      {link}
                    </div>
                  </li>
                {/each}
              </ul>
              <div class="mt-3">
                <button
                  type="button"
                  class="btn btn-secondary w-full"
                  on:click={() => copyText()}
                >
                  {copied ? "Copied!" : "Copy"}
                </button>
              </div>
            </div>
          {/if}
        </form>
      </div>
      <div class="p-4 border-t bg-background">
        {#if loading}
          <button type="submit" class="btn btn-accent btn-full" disabled
            >Generating link...</button
          >
        {:else if requestLink}
          <button type="button" class="btn btn-accent btn-full" on:click={close}
            >Request sent, close this window</button
          >
        {:else}
          <button
            type="submit"
            class="btn btn-accent btn-full"
            on:click={updateRequestUrl}>Generate link</button
          >
        {/if}
      </div>
    </div>
  </Modal>
</div>
