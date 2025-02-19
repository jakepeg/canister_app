import crypto from "$lib/crypto";
import FileTools from "$lib/file";
import type { ActorType } from "$lib/shared/actor";
import { enumIs } from "$lib/shared/enums";
import pLimit from "p-limit";
import { writable } from "svelte/store";
import type { get_alias_info_response } from "../../../../declarations/backend/backend.did";
import { VetKeyService } from "../vetkeys/encrypt";
import { File } from "./files";

export const CHUNK_SIZE = 2_000_000;

export const uploadInProgress = writable(false);

export type UploadType =
  | {
      type: "request";
      fileInfo: Extract<get_alias_info_response, { Ok: any }>["Ok"];
    }
  | {
      type: "self";
      fileName: string;
    };

export class UploadService {
  constructor(
    private actor: ActorType,
    private vetKeyService: VetKeyService,
  ) {}

  async uploadFile({
    uploadType,
    file,
    dataType,
    onStarted,
    onChunkUploaded,
    onCompleted,
    onError,
    onAborted,
  }: {
    uploadType: UploadType;
    file: globalThis.File;
    dataType: string;
    onStarted?: () => void; // Remove parameter
    onChunkUploaded?: (chunkId: number, size: number) => void;
    onCompleted?: (fileId: bigint) => void;
    onError?: () => void; // Remove parameter
    onAborted?: () => void;
  }) {
    const fileName =
      uploadType.type === "request"
        ? uploadType.fileInfo.file_name
        : uploadType.fileName;

    const fileBytes = await file.arrayBuffer();

    try {
      onStarted?.(); // Call without parameter

      let fileId: bigint;
      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
      } else {
        // Use appropriate method from your actor interface
        fileId = await this.actor.create_file({
          name: fileName,
          file_type: dataType,
        });
      }

      // Create a new instance of the File class from files.ts
      const fileToUpload = new File(fileName, fileBytes);
      const encryptedContent = await fileToUpload.encrypt(
        fileId,
        this.vetKeyService,
      );

      // Upload the encrypted content
      const content = new TextEncoder().encode(encryptedContent);

      // ... rest of your upload logic ...
    } catch (err) {
      console.error(err);
      onError?.(); // Call without parameter
    }
  }
}
