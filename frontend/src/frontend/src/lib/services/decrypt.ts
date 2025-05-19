import File from "$lib/file";
import type { ActorType } from "$lib/shared/actor";
import { formatUploadDate } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { flatten } from "$lib/shared/flatten";
import { unreachable } from "$lib/shared/unreachable";
import { writable } from "svelte/store";
import { VetkdCryptoService } from "../vetkeys/vetkdCrypto";
import type {
  AuthStateAuthenticated,
  AuthStateUnauthenticated,
} from "$lib/services/auth";

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
    this.vetkdCryptoService = new VetkdCryptoService(auth.actor);
  }

  reset() {
    this.aborted = false;
    this.progress.set(PROGRESS_INITIAL);
  }

  async decryptFile({ fileId }: { fileId: bigint }): Promise<
    | {
        name: string;
        dataType: string;
        uploadDate: string;
        contents: ArrayBuffer;
        originalMetadata: any;
      }
    | "aborted"
  > {
    this.progress.set(PROGRESS_INITIAL);

    let files = flatten(
      await Promise.all([
        this.auth.actor.get_requests(),
        this.auth.actor.get_items_shared_with_me(),
      ]),
    );

    if (this.aborted) return "aborted";

    const maybeFile = files.find((entry) => entry.id == BigInt(fileId));

    if (!maybeFile) {
      throw new Error("Error: File not found");
    }

    if (enumIs(maybeFile.file_status, "pending")) {
      throw new Error("Error: File not uploaded");
    }

    if (enumIs(maybeFile.file_status, "partially_uploaded")) {
      throw new Error("Error: File partially uploaded");
    }

    this.progress.update((v) => ({
      ...v,
      step: "downloading",
    }));

    let downloadedFile = await this.auth.actor.download_file(
      BigInt(fileId),
      0n,
    );

    if (this.aborted) return "aborted";

    if (enumIs(downloadedFile, "found_file")) {
      const totalChunks = Number(downloadedFile.found_file.num_chunks);
      this.progress.update((v) => ({
        ...v,
        totalChunks,
        currentChunk: 1,
      }));
      for (let i = 1; i < downloadedFile.found_file.num_chunks; i++) {
        const downloadedChunk = await this.auth.actor.download_file(
          BigInt(fileId),
          BigInt(i),
        );
        if (this.aborted) return "aborted";

        if (enumIs(downloadedChunk, "found_file")) {
          this.progress.update((v) => ({
            ...v,
            currentChunk: i + 1,
          }));
          const chunk = downloadedChunk.found_file.contents;

          const mergedArray = new Uint8Array(
            downloadedFile.found_file.contents.length + chunk.length,
          );
          mergedArray.set(downloadedFile.found_file.contents, 0);
          mergedArray.set(chunk, downloadedFile.found_file.contents.length);

          downloadedFile.found_file.contents = mergedArray;
        } else if (enumIs(downloadedChunk, "not_found_file")) {
          throw new Error("Error: Chunk not found");
        } else if (enumIs(downloadedChunk, "permission_error")) {
          throw new Error("Permission error");
        }
      }
      this.progress.update((v) => ({
        ...v,
        step: "decrypting",
      }));

      let decryptedFile: File;
      try {
        // Get user principal for decryption
        const userPrincipal = this.auth.authClient
          .getIdentity?.()
          .getPrincipal();
        const userPrincipalBytes = userPrincipal.toUint8Array();

        console.log("fileId", fileId);

        // Decrypt the file using vetkd
        const decryptedData = await this.vetkdCryptoService.decrypt(
          downloadedFile.found_file.contents as Uint8Array,
          userPrincipalBytes,
          fileId,
        );

        return {
          name: maybeFile.name,
          dataType: downloadedFile.found_file.file_type,
          uploadDate: formatUploadDate(
            maybeFile.file_status.uploaded.uploaded_at,
          ),
          contents: decryptedData.buffer as ArrayBuffer,
          originalMetadata: maybeFile,
        };
      } catch {
        throw new Error(
          "Failed to decrypt file: " +
            ((maybeFile.name || "unnamed file") +
              ". You may be able to access this file with a different browser, as the decryption key is stored in the browser."),
        );
      }
    } else if (enumIs(downloadedFile, "not_found_file")) {
      throw new Error("File not found");
    } else if (enumIs(downloadedFile, "permission_error")) {
      throw new Error("Permission error");
    } else if (enumIs(downloadedFile, "not_uploaded_file")) {
      throw new Error("File not uploaded");
    } else {
      unreachable(downloadedFile);
    }
  }

  abort() {
    this.aborted = true;
  }

  subscribe = this.progress.subscribe;
  set = this.progress.set;
}
