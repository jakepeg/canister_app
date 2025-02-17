import {
  CryptoService,
  createActor,
} from "@shipstone-labs/ic-vetkd-notes-client";
import type { ActorType } from "$lib/shared/actor";

export class VetKeyService {
  private cryptoService: CryptoService;

  constructor(private actor: ActorType) {
    // Initialize the CryptoService with the backend actor
    this.cryptoService = new CryptoService(actor as any);
  }

  async encrypt(fileId: bigint, data: ArrayBuffer): Promise<string> {
    const whoAmI = await this.actor.who_am_i();
    const owner =
      "known_user" in whoAmI ? whoAmI.known_user.username : "unknown";
    return await this.cryptoService.encryptWithNoteKey(
      fileId,
      owner,
      new TextDecoder().decode(data),
    );
  }

  async decrypt(fileId: bigint, encryptedData: string): Promise<Uint8Array> {
    const whoAmI = await this.actor.who_am_i();
    const owner =
      "known_user" in whoAmI ? whoAmI.known_user.username : "unknown";
    const decryptedString = await this.cryptoService.decryptWithNoteKey(
      fileId,
      owner,
      encryptedData,
    );
    return new TextEncoder().encode(decryptedString);
  }
}
