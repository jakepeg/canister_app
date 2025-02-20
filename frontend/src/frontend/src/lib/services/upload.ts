import crypto from "$lib/crypto";
import FileTools from "$lib/file";
import type { ActorType } from "$lib/shared/actor";
import { enumIs } from "$lib/shared/enums";
import pLimit from "p-limit";
import { writable } from "svelte/store";
import type { get_alias_info_response } from "../../../../declarations/backend/backend.did";
import { VetKeyService } from "../vetkeys/encrypt";
import CustomFile from "$lib/file";

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

// export class UploadService {
//   aborted = false;

//   constructor(private actor: ActorType) {}

//   async uploadFile({
//     uploadType,
//     file,
//     onChunkUploaded = () => {},
//     onCompleted = () => {},
//     onError = () => {},
//     dataType,
//     onStarted = () => {},
//     onAborted = () => {},
//   }: {
//     uploadType: UploadType;
//     file: File;
//     dataType: string;
//     onStarted?: (totalSizeBytes: number) => void;
//     onChunkUploaded?: (chunkId: number, size: number) => void;
//     onCompleted?: (file_id: bigint) => void;
//     onError?: (message: string) => void;
//     onAborted?: () => void;
//   }) {
//     const userPublicKey =
//       uploadType.type === "request"
//         ? (uploadType.fileInfo.user.public_key as Uint8Array).buffer
//         : new Uint8Array(await crypto.getLocalUserPublicKey());

//     const fileName =
//       uploadType.type === "request"
//         ? uploadType.fileInfo.file_name
//         : uploadType.fileName;

//     const fileBytes = await file.arrayBuffer();
//     let fileToEncrypt = FileTools.fromUnencrypted(fileName, fileBytes);
//     const encryptedFileKey =
//       await fileToEncrypt.getEncryptedFileKey(userPublicKey);

//     const encFile = await fileToEncrypt.encrypt();
//     const content = new Uint8Array(encFile);

//     if (content.length > 100 * 1024 * 1024) {
//       onError(
//         "File size is limited to 100MiB in this PoC\n(larger files could be supported in a production version).",
//       );
//       return;
//     }

//     // Split file into chunks of 2MB.
//     const numChunks = Math.ceil(content.length / CHUNK_SIZE);

//     try {
//       onStarted(content.length);

//       const firstChunk = content.subarray(0, CHUNK_SIZE);
//       let fileId: bigint = 0n;
//       if (uploadType.type === "request") {
//         fileId = uploadType.fileInfo.file_id;
//         const res = await this.actor.upload_file({
//           file_id: fileId,
//           file_content: firstChunk,
//           owner_key: new Uint8Array(encryptedFileKey),
//           file_type: dataType,
//           num_chunks: BigInt(numChunks),
//         });

//         if (enumIs(res, "Err")) {
//           onError(
//             "An error occurred while uploading the file. Please try again.",
//           );
//           return;
//         }
//       } else {
//         fileId = await this.actor.upload_file_atomic({
//           content: firstChunk,
//           owner_key: new Uint8Array(encryptedFileKey),
//           name: fileName,
//           file_type: dataType,
//           num_chunks: BigInt(numChunks),
//         });
//       }

//       onChunkUploaded(0, firstChunk.length);

//       if (this.aborted) {
//         onAborted();
//         return;
//       }

//       await this.uploadChunks(content, fileId, onChunkUploaded);

//       if (this.aborted) {
//         onAborted();
//         return;
//       }

//       onCompleted(fileId);
//     } catch (err) {
//       console.error(err);
//       onError("An error occurred while uploading the file. Please try again.");
//     }
//   }

//   private async uploadChunks(
//     content: Uint8Array,
//     fileId: bigint,
//     onChunkUploaded: (chunkId: number, size: number) => void,
//   ) {
//     const numChunks = Math.ceil(content.length / CHUNK_SIZE);

//     // Create upload pool, supporting upto 5 parallel uploads.
//     const uploadPool = pLimit(5);

//     // Prepare upload requests.
//     const uploadRequests = Array.from(
//       { length: numChunks - 1 },
//       (_, i) => i + 1,
//     ).map((i) =>
//       uploadPool(async () => {
//         if (this.aborted) {
//           return;
//         }
//         const chunk = content.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
//         await this.actor.upload_file_continue({
//           file_id: fileId,
//           contents: chunk,
//           chunk_id: BigInt(i),
//         });
//         onChunkUploaded(i, chunk.length);
//       }),
//     );

//     await Promise.all(uploadRequests);
//   }

//   async abort() {
//     this.aborted = true;
//   }
// }

export class UploadService {
  aborted = false;

  constructor(
    private actor: ActorType,
    private vetKeyService: VetKeyService,
  ) {}

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
    const fileName =
      uploadType.type === "request"
        ? uploadType.fileInfo.file_name
        : uploadType.fileName;

    try {
      const fileBytes = await file.arrayBuffer();

      let fileId: bigint;
      // Get file ID from request if it exists
      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
      }

      // Create our File instance
      const fileToUpload = File.fromUnencrypted(fileName, fileBytes);

      // For request type, we already have the fileId so we can encrypt
      // For self type, we'll need to do the first upload to get the fileId
      let encryptedContent: string;
      let content: Uint8Array;

      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
        encryptedContent = await fileToUpload.encrypt(
          fileId,
          this.vetKeyService,
        );
        content = new TextEncoder().encode(encryptedContent);
      } else {
        // For self uploads, we need to do the initial upload to get the fileId
        const tempContent = new TextEncoder().encode(
          new TextDecoder().decode(fileBytes),
        );
        content = tempContent; // Temporary unencrypted for first chunk
      }

      if (content.length > 100 * 1024 * 1024) {
        onError(
          "File size is limited to 100MiB in this PoC\n(larger files could be supported in a production version).",
        );
        return;
      }

      const numChunks = Math.ceil(content.length / CHUNK_SIZE);
      onStarted(content.length);

      const firstChunk = content.subarray(0, CHUNK_SIZE);

      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
        const res = await this.actor.upload_file({
          file_id: fileId,
          file_content: firstChunk,
          owner_key: new Uint8Array(encryptedFileKey),
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });

        if (enumIs(res, "Err")) {
          onError(
            "An error occurred while uploading the file. Please try again.",
          );
          return;
        }
      } else {
        // Use upload_file_atomic for self uploads to get the fileId
        fileId = await this.actor.upload_file_atomic({
          content: firstChunk,
          name: fileName,
          owner_key: new Uint8Array(encryptedFileKey),
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });

        // Now that we have the fileId, we can encrypt the content for remaining chunks
        encryptedContent = await fileToUpload.encrypt(
          fileId,
          this.vetKeyService,
        );
        content = new TextEncoder().encode(encryptedContent);
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
    const uploadPool = pLimit(5);

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
