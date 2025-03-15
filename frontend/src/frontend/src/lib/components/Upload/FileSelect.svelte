<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import FileDropzone from "svelte-file-dropzone";

  export let type: "self" | "request" = "self";
  export let disabled: boolean = false;
  export let fileName: string = "";
  export let onFileSelected: (file: File | null) => void = () => {};

  let fileNameAutoFilled: boolean = false;
  let selectedFile: File | null = null;

  const dispatch = createEventDispatcher<{ "file-selected": File | null }>();

  function handleFileSelection(event) {
    const files = event.detail?.acceptedFiles || [];

    if (files.length === 0) {
      console.warn("No files selected");
      return;
    }

    selectedFile = files[0];

    if (type === "self" && (fileNameAutoFilled || fileName.trim() === "")) {
      fileNameAutoFilled = true;
      fileName = selectedFile?.name || "";
    }

    dispatch("file-selected", selectedFile);
    onFileSelected(selectedFile);
  }
</script>

{#if type === "self"}
  <div>
    <label for="fileName" class="input-label">File Name</label>
    <input
      type="text"
      required
      class="input"
      id="fileName"
      name="fileName"
      placeholder="File name"
      {disabled}
      bind:value={fileName}
      on:input={() => (fileNameAutoFilled = false)}
    />
  </div>
{/if}

<!-- Drag & Drop Zone -->
<FileDropzone on:drop={handleFileSelection} {disabled}>
  <div
    class="
      flex items-center justify-center
      p-6 border-2 border-dashed border-gray-300
      rounded-lg cursor-pointer hover:border-gray-400
      text-gray-500"
  >
    <p>Drag & drop files here or click to select</p>
  </div>
</FileDropzone>
