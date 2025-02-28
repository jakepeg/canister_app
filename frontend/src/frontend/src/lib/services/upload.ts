import crypto from "$lib/crypto";
// import FileTools from "$lib/file";
// import type { ActorType } from "$lib/shared/actor";
import type {
  AuthStateAuthenticated,
  AuthStateUnauthenticated,
} from "$lib/services/auth";
import { enumIs } from "$lib/shared/enums";
import pLimit from "p-limit";
import { writable } from "svelte/store";
import type { get_alias_info_response } from "../../../../declarations/backend/backend.did";
export const CHUNK_SIZE = 2_000_000;
import { VetkdCryptoService } from "../vetkeys/vetkdCrypto";

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
  private vetkdCryptoService: VetkdCryptoService;

  constructor(private auth: AuthStateAuthenticated | AuthStateUnauthenticated) {
    this.vetkdCryptoService = new VetkdCryptoService(auth.actor);
  }

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
    // Get user principal for encryption
    const userId =
      uploadType.type === "request"
        ? uploadType.fileInfo.user.ic_principal
        : this.auth.authClient.getIdentity?.().getPrincipal();

    const userPrincipalBytes = userId.toUint8Array();

    // Redundant, moving to vetkd
    // const userPublicKey =
    //   uploadType.type === "request"
    //     ? (uploadType.fileInfo.user.public_key as Uint8Array).buffer
    //     : new Uint8Array(await crypto.getLocalUserPublicKey());

    const fileName =
      uploadType.type === "request"
        ? uploadType.fileInfo.file_name
        : uploadType.fileName;

    console.log("fileName: ", fileName);

    // Read file as ArrayBuffer
    const fileBytes = await file.arrayBuffer();

    // Redundant, moving to vetkd
    // let fileToEncrypt = FileTools.fromUnencrypted(fileName, fileBytes);
    // console.log("fileToEncrypt done");

    // const encryptedFileKey =
    //   await fileToEncrypt.getEncryptedFileKey(userPublicKey);

    // console.log("encryptedFileKey done");

    // const encFile = await fileToEncrypt.encrypt();
    // console.log("encFile done");
    // const content = new Uint8Array(encFile);
    // console.log("content done");

    onStarted(0); // Show start progress while encrypting
    const encryptedData = await this.vetkdCryptoService.encrypt(
      fileBytes,
      userPrincipalBytes,
    );

    if (encryptedData.length > 100 * 1024 * 1024) {
      onError(
        "File size is limited to 100MiB in this PoC\n(larger files could be supported in a production version).",
      );
      return;
    }

    // Split file into chunks of 2MB.
    const numChunks = Math.ceil(encryptedData.length / CHUNK_SIZE);

    try {
      onStarted(encryptedData.length);
      console.log("onStarted done");

      const firstChunk = encryptedData.subarray(0, CHUNK_SIZE);
      console.log("firstChunk done");
      let fileId: bigint = 0n;

      // Get the vetkd public key to use as the owner key
      const publicKeyResponse = await this.auth.actor.vetkd_public_key();
      if (!publicKeyResponse || "Err" in publicKeyResponse) {
        throw new Error("Error getting public key");
      }
      const publicKey = publicKeyResponse.Ok as Uint8Array;

      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
        console.log("fileId for request: ", fileId);
        const res = await this.auth.actor.upload_file({
          file_id: fileId,
          file_content: firstChunk,
          owner_key: publicKey,
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
        fileId = await this.auth.actor.upload_file_atomic({
          content: firstChunk,
          owner_key: publicKey,
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

      await this.uploadChunks(encryptedData, fileId, onChunkUploaded);

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
        await this.auth.actor.upload_file_continue({
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
