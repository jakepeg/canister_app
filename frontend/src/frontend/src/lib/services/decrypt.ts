// ic-docutrack/frontend/src/frontend/src/lib/services/decrypt.ts
import File from "$lib/file"; // Assuming this is a local utility, not from declarations
import type { ActorType } from "$lib/shared/actor";
import { formatUploadDate } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { flatten } from "$lib/shared/flatten"; // This might need re-evaluation for how file metadata is fetched
import { unreachable } from "$lib/shared/unreachable";
import { writable } from "svelte/store";
import { VetkdCryptoService } from "../vetkeys/vetkdCrypto";
import type {
  AuthStateAuthenticated,
  AuthStateUnauthenticated,
} from "$lib/services/auth";
import type {
  public_item_metadata,
  download_file_chunk_response,
  item_id,
} from "../../../../declarations/backend/backend.did"; // Added types

type ProgressStore = {
  step: "initializing" | "downloading" | "decrypting";
  totalChunks: number;
  currentChunk: number;
};

const PROGRESS_INITIAL: ProgressStore = {
  step: "initializing",
  totalChunks: 0,
  currentChunk: 0,
};

export class DecryptService {
  aborted = false;
  progress = writable<ProgressStore>(PROGRESS_INITIAL);
  private vetkdCryptoService: VetkdCryptoService;

  constructor(private auth: AuthStateAuthenticated | AuthStateUnauthenticated) {
    // Ensure actor is available, especially if auth state can be unauthenticated here.
    // For decryption, usually an authenticated actor is needed.
    if (!this.auth.actor) {
      throw new Error(
        "Actor is not available for DecryptService. Authentication might be required.",
      );
    }
    this.vetkdCryptoService = new VetkdCryptoService(this.auth.actor);
  }

  reset() {
    this.aborted = false;
    this.progress.set(PROGRESS_INITIAL);
  }

  // Helper to get file metadata. This needs to be adapted to the new hierarchical system.
  // It might involve looking up the item by ID from a cache or fetching its details.
  // For now, it tries to find it in a combined list of 'requests' and 'shared_with_me',
  // but ideally, it should fetch metadata for a specific item_id if not already available.
  private async getFileMetadata(
    fileId: item_id,
  ): Promise<public_item_metadata | undefined> {
    if (this.aborted) return undefined;
    console.log(
      `DecryptService: Fetching metadata directly for fileId: ${fileId}`,
    );
    try {
      const result = await this.auth.actor.get_item_metadata_by_id(fileId);
      if (enumIs(result, "Ok")) {
        return result.Ok;
      } else {
        console.error(
          `Failed to get metadata for fileId ${fileId}: ${result.Err}`,
        );
        // Optionally, throw the error message from result.Err to be displayed to the user
        throw new Error(`Failed to load file details: ${result.Err}`);
        // return undefined;
      }
    } catch (e) {
      console.error(`Error in getFileMetadata for fileId ${fileId}:`, e);
      if (e instanceof Error) throw e; // Re-throw if it's already an error
      throw new Error(
        `Network or canister error fetching metadata for file ID ${fileId}.`,
      );
      // return undefined;
    }
  }

  async decryptFile({ fileId }: { fileId: item_id }): Promise<
    | {
        name: string;
        dataType: string;
        uploadDate: string;
        contents: ArrayBuffer;
        originalMetadata: public_item_metadata; // Ensure this is the correct metadata type
      }
    | "aborted"
  > {
    this.progress.set(PROGRESS_INITIAL);

    const maybeFile = await this.getFileMetadata(fileId);

    if (this.aborted) return "aborted";

    if (!maybeFile) {
      throw new Error(`Error: File metadata not found for ID ${fileId}.`);
    }

    // Check if it's a file
    // Assuming item_type is like { File: null } or { Folder: null }
    if (!maybeFile.item_type || !("File" in maybeFile.item_type)) {
      throw new Error("Error: The specified item is not a file.");
    }

    // Check if it has size (implies it's uploaded)
    let hasSize = false;
    const sizeOpt = maybeFile.size; // size is: [] | [bigint]

    if (Array.isArray(sizeOpt) && sizeOpt.length > 0) {
      // Now we know sizeOpt is [bigint]
      const actualSize = sizeOpt[0];
      if (typeof actualSize === "bigint" && actualSize > 0n) {
        hasSize = true;
      }
    }
    // The case `else if (typeof maybeFile.size === "bigint" ...)` is generally not needed
    // if the types are strictly from Candid `opt nat64`.

    if (!hasSize) {
      throw new Error(
        "Error: File not uploaded or has no content (size is missing, zero, or invalid).",
      );
    }

    this.progress.update((v) => ({
      ...v,
      step: "downloading",
    }));

    // Download first chunk
    let downloadedFileResponse = await this.auth.actor.download_file_chunk(
      fileId, // Already a bigint
      0n,
    );

    if (this.aborted) return "aborted";

    // Use the new variant 'found_file_chunk'
    if (enumIs(downloadedFileResponse, "found_file_chunk")) {
      const firstChunkData = downloadedFileResponse.found_file_chunk;
      const totalChunks = Number(firstChunkData.num_chunks);
      let accumulatedContents = new Uint8Array(firstChunkData.contents); // Start with the first chunk

      this.progress.update((v) => ({
        ...v,
        totalChunks,
        currentChunk: 1,
      }));

      // Download subsequent chunks if any
      for (let i = 1; i < totalChunks; i++) {
        const downloadedChunkResponse =
          await this.auth.actor.download_file_chunk(fileId, BigInt(i));

        if (this.aborted) return "aborted";

        if (enumIs(downloadedChunkResponse, "found_file_chunk")) {
          this.progress.update((v) => ({
            ...v,
            currentChunk: i + 1,
          }));
          const chunkData = downloadedChunkResponse.found_file_chunk.contents;

          const newAccumulatedContents = new Uint8Array(
            accumulatedContents.length + chunkData.length,
          );
          newAccumulatedContents.set(accumulatedContents, 0);
          newAccumulatedContents.set(
            new Uint8Array(chunkData),
            accumulatedContents.length,
          );
          accumulatedContents = newAccumulatedContents;
        } else if (enumIs(downloadedChunkResponse, "chunk_not_found")) {
          throw new Error(`Error: Chunk ${i} not found for file ID ${fileId}.`);
        } else if (enumIs(downloadedChunkResponse, "permission_error")) {
          throw new Error(
            `Permission error downloading chunk ${i} for file ID ${fileId}.`,
          );
        } else if (enumIs(downloadedChunkResponse, "not_a_file")) {
          throw new Error(
            `Item with ID ${fileId} is not a file (error during chunk download).`,
          );
        } else if (enumIs(downloadedChunkResponse, "not_found_item")) {
          throw new Error(
            `File with ID ${fileId} not found (error during chunk download).`,
          );
        } else if (enumIs(downloadedChunkResponse, "not_uploaded_file")) {
          throw new Error(
            `File with ID ${fileId} is not fully uploaded (error during chunk download).`,
          );
        } else {
          unreachable(downloadedChunkResponse); // Should cover all variants
        }
      }

      this.progress.update((v) => ({
        ...v,
        step: "decrypting",
      }));

      // let decryptedFile: File; // File type might be an internal utility
      try {
        const identity = this.auth.authClient?.getIdentity?.();
        if (!identity) {
          throw new Error("User identity not available for decryption.");
        }
        const userPrincipalBytes = identity.getPrincipal().toUint8Array();

        console.log("Decrypting fileId:", fileId);

        const decryptedData = await this.vetkdCryptoService.decrypt(
          accumulatedContents, // This is already a Uint8Array
          userPrincipalBytes,
          fileId,
        );

        return {
          name: maybeFile.name,
          dataType: firstChunkData.file_type, // Use file_type from the first chunk data
          uploadDate: formatUploadDate(maybeFile.modified_at),
          contents: decryptedData.buffer as ArrayBuffer, // Assuming decrypt returns { buffer: ArrayBuffer }
          originalMetadata: maybeFile,
        };
      } catch (e) {
        console.error("Decryption failed:", e);
        throw new Error(
          `Failed to decrypt file: ${maybeFile.name || "unnamed file"}. ` +
            (e instanceof Error ? e.message : "Decryption error.") +
            " You may be able to access this file with a different browser, as the decryption key is stored in the browser.",
        );
      }
    } else if (enumIs(downloadedFileResponse, "not_found_item")) {
      throw new Error(
        `File not found (ID: ${fileId}) when attempting to download first chunk.`,
      );
    } else if (enumIs(downloadedFileResponse, "permission_error")) {
      throw new Error(
        `Permission error when attempting to download first chunk of file ID ${fileId}.`,
      );
    } else if (enumIs(downloadedFileResponse, "not_uploaded_file")) {
      throw new Error(
        `File (ID: ${fileId}) is not uploaded (error on first chunk download).`,
      );
    } else if (enumIs(downloadedFileResponse, "not_a_file")) {
      throw new Error(`Item with ID ${fileId} is not a file.`);
    } else if (enumIs(downloadedFileResponse, "chunk_not_found")) {
      // Unlikely for chunk 0, but handle
      throw new Error(
        `Chunk 0 not found for file ID ${fileId}. This indicates an issue with the file upload.`,
      );
    } else {
      unreachable(downloadedFileResponse);
    }
  }

  abort() {
    this.aborted = true;
  }

  // Expose subscribe and set if needed, though usually progress is just subscribed to.
  subscribe = this.progress.subscribe;
  set = this.progress.set;
}
