import crypto from "$lib/crypto";
import FileTools from "$lib/file";
import type { ActorType } from "$lib/shared/actor";
import { enumIs } from "$lib/shared/enums";
import pLimit from "p-limit";
import { writable } from "svelte/store";
import type { get_alias_info_response } from "../../../../declarations/backend/backend.did";
export const CHUNK_SIZE = 2_000_000;
import {
  createActor,
  CryptoService,
} from "@shipstone-labs/ic-vetkd-notes-client";

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
  aborted = false;

  constructor(private actor: ActorType) {}

  async uploadFile({
    uploadType,
    file,
    onChunkUploaded = () => {},
    onCompleted = () => {},
    onError = () => {},
    dataType,
    onStarted = () => {},
    onAborted = () => {},
  }: {
    uploadType: UploadType;
    file: File;
    dataType: string;
    onStarted?: (totalSizeBytes: number) => void;
    onChunkUploaded?: (chunkId: number, size: number) => void;
    onCompleted?: (file_id: bigint) => void;
    onError?: (message: string) => void;
    onAborted?: () => void;
  }) {
    const userPublicKey =
      uploadType.type === "request"
        ? (uploadType.fileInfo.user.public_key as Uint8Array).buffer
        : new Uint8Array(await crypto.getLocalUserPublicKey());

    const fileName =
      uploadType.type === "request"
        ? uploadType.fileInfo.file_name
        : uploadType.fileName;

    console.log("fileName: ", fileName);

    const fileBytes = await file.arrayBuffer();
    let fileToEncrypt = FileTools.fromUnencrypted(fileName, fileBytes);
    console.log("fileToEncrypt done");

    const encryptedFileKey =
      await fileToEncrypt.getEncryptedFileKey(userPublicKey);

    console.log("encryptedFileKey done");

    const encFile = await fileToEncrypt.encrypt();
    console.log("encFile done");
    const content = new Uint8Array(encFile);
    console.log("content done");

    if (content.length > 100 * 1024 * 1024) {
      onError(
        "File size is limited to 100MiB in this PoC\n(larger files could be supported in a production version).",
      );
      return;
    }

    // Split file into chunks of 2MB.
    const numChunks = Math.ceil(content.length / CHUNK_SIZE);
    console.log("numChunks done");

    try {
      onStarted(content.length);
      console.log("onStarted done");

      const firstChunk = content.subarray(0, CHUNK_SIZE);
      console.log("firstChunk done");
      let fileId: bigint = 0n;
      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
        console.log("fileId for request: ", fileId);
        const res = await this.actor.upload_file({
          file_id: fileId,
          file_content: firstChunk,
          owner_key: new Uint8Array(encryptedFileKey),
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });
        console.log("res done for request");

        if (enumIs(res, "Err")) {
          onError(
            "An error occurred while uploading the file. Please try again.",
          );
          return;
        }
      } else {
        // shipstone actor
        // const actor = createActor();
        // // Create new note and get its ID
        // const testFileId = await actor.create_note();

        // console.log("fileId for self: ", testFileId);

        fileId = await this.actor.upload_file_atomic({
          content: firstChunk,
          owner_key: new Uint8Array(encryptedFileKey),
          name: fileName,
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });
        // console.log("fileId for self: ", fileId);
      }

      onChunkUploaded(0, firstChunk.length);

      if (this.aborted) {
        onAborted();
        return;
      }

      await this.uploadChunks(content, fileId, onChunkUploaded);

      if (this.aborted) {
        onAborted();
        return;
      }

      onCompleted(fileId);
    } catch (err) {
      console.error(err);
      onError("An error occurred while uploading the file. Please try again.");
    }
  }

  private async uploadChunks(
    content: Uint8Array,
    fileId: bigint,
    onChunkUploaded: (chunkId: number, size: number) => void,
  ) {
    const numChunks = Math.ceil(content.length / CHUNK_SIZE);

    // Create upload pool, supporting upto 5 parallel uploads.
    const uploadPool = pLimit(5);

    // Prepare upload requests.
    const uploadRequests = Array.from(
      { length: numChunks - 1 },
      (_, i) => i + 1,
    ).map((i) =>
      uploadPool(async () => {
        if (this.aborted) {
          return;
        }
        const chunk = content.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
        await this.actor.upload_file_continue({
          file_id: fileId,
          contents: chunk,
          chunk_id: BigInt(i),
        });
        onChunkUploaded(i, chunk.length);
      }),
    );

    await Promise.all(uploadRequests);
  }

  async abort() {
    this.aborted = true;
  }
}
