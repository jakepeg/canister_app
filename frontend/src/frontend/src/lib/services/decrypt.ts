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
    // Option 1: If FileList.svelte or another service keeps a cache of items, use that.
    // Option 2: Fetch directly if not found (less efficient if called often).
    // For this example, we'll stick to the provided logic of searching a pre-fetched list,
    // but acknowledge this is a simplification and might not reflect how a file's metadata
    // should be obtained in a details view (it should ideally be passed or fetched by ID specifically).

    // The current logic fetches all requests and shared items.
    // This might not include items the user owns but hasn't explicitly shared or requested.
    // A better approach for a details page would be to have an endpoint like `get_item_metadata(item_id)`.
    // Lacking that, this is an approximation.
    console.warn(
      "getFileMetadata in DecryptService is using a potentially incomplete list of files (requests & shared). For a robust details view, consider fetching item metadata directly by ID.",
    );

    // To get ANY item (owned, shared, etc.), we might need a different approach
    // or assume the FileList has already loaded relevant items into a store.
    // For now, let's assume the item is either a request or shared with me, or was listed in root.
    // This part is tricky without knowing how 'maybeFile' is guaranteed to be found.
    // Let's simulate a more direct fetch or rely on a broader list if available.
    // The most robust way is to have an endpoint `get_item_metadata(id: item_id) -> (opt public_item_metadata)`.
    // Since we don't have that, we'll use the existing logic which might fail if the file is just "owned" and in a subfolder.

    // This fetching logic is problematic for general file access.
    // The fileId comes from the URL, it could be any file the user has access to.
    // The `list_folder_contents` with the item's parent_id would be more appropriate to get siblings,
    // and one of those siblings would be the item itself.
    // Or, `list_folder_contents` could be modified to optionally take an item_id and return just that item's metadata.
    // For now, the code uses a combined list, which is kept.

    let files: public_item_metadata[] = [];
    try {
      const requests = await this.auth.actor.get_requests(); // My pending items
      const sharedItems = await this.auth.actor.get_items_shared_with_me();
      // To find an owned item, we'd ideally list its parent folder.
      // This is a simplification:
      const rootItemsResponse = await this.auth.actor.list_folder_contents([]);
      let ownedRootItems: public_item_metadata[] = [];
      if (enumIs(rootItemsResponse, "Ok")) {
        ownedRootItems = rootItemsResponse.Ok;
      }

      // This is a very broad way to find an item and might be inefficient or incomplete.
      const combined = [...requests, ...sharedItems, ...ownedRootItems];
      const uniqueFiles = new Map<bigint, public_item_metadata>();
      combined.forEach((f) => {
        if (!uniqueFiles.has(f.id)) {
          uniqueFiles.set(f.id, f);
        }
      });
      files = Array.from(uniqueFiles.values());
    } catch (e) {
      console.error("Failed to fetch file lists for metadata lookup:", e);
      return undefined;
    }

    if (this.aborted) return undefined; // Check for abort

    return files.find((entry) => entry.id === fileId);
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
