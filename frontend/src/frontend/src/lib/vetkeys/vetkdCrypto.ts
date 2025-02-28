import * as vetkd from "ic-vetkd-utils";
import type { ActorType } from "$lib/shared/actor";

export class VetkdCryptoService {
  constructor(private actor: ActorType) {}

  async encrypt(
    data: ArrayBuffer,
    userPrincipalBytes: Uint8Array,
  ): Promise<Uint8Array> {
    try {
      // Get public key from the backend
      const publicKeyResponse = await this.actor.vetkd_public_key();
      if (!publicKeyResponse || "Err" in publicKeyResponse) {
        throw new Error(
          "Error getting public key: " +
            ("Err" in publicKeyResponse
              ? publicKeyResponse.Err
              : "empty response"),
        );
      }
      const publicKey = publicKeyResponse.Ok as Uint8Array;

      // Generate a random seed
      const seed = window.crypto.getRandomValues(new Uint8Array(32));

      // Transform data to Uint8Array
      const encodedMessage = new Uint8Array(data);

      // Encrypt the data using vetkd IBE
      const encryptedData = vetkd.IBECiphertext.encrypt(
        publicKey,
        userPrincipalBytes,
        encodedMessage,
        seed,
      );

      return encryptedData.serialize();
    } catch (error) {
      console.error("Encryption error:", error);
      throw error;
    }
  }

  async decrypt(
    encryptedData: Uint8Array,
    userPrincipalBytes: Uint8Array,
  ): Promise<Uint8Array> {
    try {
      // Generate a random seed for the transport secret key
      const seed = window.crypto.getRandomValues(new Uint8Array(32));

      // Initialize the transport secret key
      const transportSecretKey = new vetkd.TransportSecretKey(seed);

      // Get public key from the backend
      const publicKeyResponse = await this.actor.vetkd_public_key();
      if (!publicKeyResponse || "Err" in publicKeyResponse) {
        throw new Error(
          "Error getting public key: " +
            ("Err" in publicKeyResponse
              ? publicKeyResponse.Err
              : "empty response"),
        );
      }
      const publicKey = publicKeyResponse.Ok as Uint8Array;

      // Get encrypted key from the backend
      const privateKeyResponse = await this.actor.vetkd_encrypted_key(
        transportSecretKey.public_key(),
      );
      if (!privateKeyResponse || "Err" in privateKeyResponse) {
        throw new Error(
          "Error getting encrypted key: " +
            ("Err" in privateKeyResponse
              ? privateKeyResponse.Err
              : "empty response"),
        );
      }
      const encryptedKey = privateKeyResponse.Ok as Uint8Array;

      // Decrypt the key with the transport secret
      const key = transportSecretKey.decrypt(
        encryptedKey,
        publicKey,
        userPrincipalBytes,
      );

      // Deserialize and decrypt the data
      const ibeCiphertext = vetkd.IBECiphertext.deserialize(encryptedData);
      return ibeCiphertext.decrypt(key);
    } catch (error) {
      console.error("Decryption error:", error);
      throw error;
    }
  }
}
