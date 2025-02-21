<script lang="ts">
  import { onMount } from "svelte";
  import { VetKeyService } from "$lib/vetkeys/encrypt";
  import { createActor } from "@shipstone-labs/ic-vetkd-notes-client";
  import type {
    BackendActor,
    EncryptedNote,
  } from "@shipstone-labs/ic-vetkd-notes-client";

  let file: File | null = null;
  let notes: EncryptedNote[] = [];
  let decryptedContent: Uint8Array | null = null;
  let error: string | null = null;
  let actor: BackendActor = createActor();
  let vetKeyService: VetKeyService = new VetKeyService(actor);

  onMount(async () => await refreshNotes());

  async function refreshNotes() {
    try {
      notes = await actor.get_notes();
    } catch (err) {
      error = "Failed to load files";
    }
  }

  async function handleUpload() {
    if (!file) return;

    try {
      // const noteId = await actor.create_note();
      const noteId = 1n;
      const fileData = new Uint8Array(await file.arrayBuffer());
      const encrypted = await vetKeyService.encrypt(noteId, fileData.buffer);
      await actor.update_note(noteId, file.name, encrypted);
      await refreshNotes();
    } catch (err) {
      error = "Upload failed: " + err.message;
    }
  }

  async function handleDecrypt(noteId: bigint) {
    try {
      const note = notes.find((n) => n.id === noteId);
      if (!note) throw Error("File not found");

      decryptedContent = await vetKeyService.decrypt(
        noteId,
        note.encrypted_text,
      );
    } catch (err) {
      error = "Decryption failed: " + err.message;
    }
  }
</script>

<div class="container">
  <div class="upload-section">
    <input
      type="file"
      on:change={(e) => (file = e.target.files?.[0] || null)}
    />
    <button on:click={handleUpload}>Encrypt & Upload</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="file-list">
    <h2>Encrypted Files</h2>
    <ul>
      {#each notes as note (note.id)}
        <li>
          {note.encrypted_text.substring(0, 20)}...
          <button on:click={() => handleDecrypt(note.id)}>Decrypt</button>
        </li>
      {/each}
    </ul>
  </div>

  {#if decryptedContent}
    <div class="preview">
      <h3>Decrypted Content</h3>
      <pre>{new TextDecoder().decode(decryptedContent)}</pre>
    </div>
  {/if}
</div>

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  .upload-section {
    margin-bottom: 2rem;
    display: flex;
    gap: 1rem;
  }

  .file-list ul {
    list-style: none;
    padding: 0;
  }

  .file-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    border-bottom: 1px solid #eee;
  }

  .error {
    color: red;
    margin: 1rem 0;
  }

  .preview pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 4px;
  }
</style>
