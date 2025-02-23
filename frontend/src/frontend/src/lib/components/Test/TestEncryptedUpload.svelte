<script lang="ts">
  import { onMount } from "svelte";
  import * as vetkd from "ic-vetkd-utils/ic_vetkd_utils";
  import type {
    AuthStateAuthenticated,
    AuthStateUnauthenticated,
  } from "$lib/services/auth";

  let auth: AuthStateAuthenticated | AuthStateUnauthenticated;
  let file: File | null = null;
  let decryptedContent: Uint8Array | null = null;
  let error: string | null = null;

  onMount(async () => await refreshNotes());

  async function refreshNotes() {
    try {
      // TODO
      return;
    } catch (err) {
      error = "Failed to load files";
    }
  }

  async function handleUpload() {
    try {
      if (!file) {
        throw new Error("No file selected");
      }
      // TODO
      // 1. Encrypt file
      const seed = window.crypto.getRandomValues(new Uint8Array(32));
      const fileBuffer = await file.arrayBuffer();
      const encodedMessage = new Uint8Array(fileBuffer);
      const encryptedFile = vetkd.IBECiphertext.encrypt();
      // 2. Upload encrypted file
      return;
    } catch (err) {
      error = "Upload failed: " + err;
    }
  }

  async function handleDecrypt(noteId: bigint) {
    try {
      // TODO
      // 1. Decrypt file
      // 2. Understand where to get derived_public_key_bytes (is it the public_key method?)
      // 3. Understand where to get derivation_id (it was suggested to use the caller)
      const seed = window.crypto.getRandomValues(new Uint8Array(32));
      const secretKey = new vetkd.TransportSecretKey(seed);
      const publicKey = secretKey.public_key();
      const decryptedFile = secretKey.decrypt();
      return;
    } catch (err) {
      error = "Decryption failed: " + err;
    }
  }

  function handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (files && files.length > 0) {
      file = files[0];
    }
  }
</script>

<div class="container">
  <div class="upload-section">
    <input type="file" on:change={handleFileChange} />
    <button on:click={handleUpload}>Encrypt & Upload</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="file-list">
    <h2>Encrypted Files</h2>
    <ul>
      <!-- {#each notes as note (note.id)}
        <li>
          {note.encrypted_text.substring(0, 20)}...
          <button on:click={() => handleDecrypt(note.id)}>Decrypt</button>
        </li>
      {/each} -->
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
