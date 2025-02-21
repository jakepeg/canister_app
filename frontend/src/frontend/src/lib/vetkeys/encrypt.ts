import {
  CryptoService,
  createActor,
} from "@shipstone-labs/ic-vetkd-notes-client";
// import type { ActorType } from "$lib/shared/actor";
import type {
  BackendActor,
  EncryptedNote,
} from "@shipstone-labs/ic-vetkd-notes-client";

export class VetKeyService {
  private cryptoService: CryptoService;

  constructor(private actor: BackendActor) {
    // Initialize the CryptoService with the backend actor
    this.cryptoService = new CryptoService(actor as any);
  }

  async encrypt(fileId: bigint, data: ArrayBuffer): Promise<string> {
    const owner = await this.actor.whoami();
    return await this.cryptoService.encryptWithNoteKey(
      fileId,
      owner,
      new TextDecoder().decode(data),
    );
  }

  async decrypt(fileId: bigint, encryptedData: string): Promise<Uint8Array> {
    const owner = await this.actor.whoami();
    const decryptedString = await this.cryptoService.decryptWithNoteKey(
      fileId,
      owner,
      encryptedData,
    );
    return new TextEncoder().encode(decryptedString);
  }
}
