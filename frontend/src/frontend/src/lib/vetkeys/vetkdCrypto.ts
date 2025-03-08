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
    fileId: bigint,
  ): Promise<Uint8Array> {
    try {
      // First, check if this is a shared file by checking if file is in shared_files
      const sharedFiles = await this.actor.get_shared_files();
      const isSharedFile = sharedFiles.some((file) => file.file_id === fileId);

      console.log("Is this a shared file?", isSharedFile);

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
      console.log("publicKey: ", publicKey);

      console.log("fileId: ", fileId);
      console.log("[fileId]: ", [fileId]);

      // For shared files, always get the owner's principal
      let principalToUse = userPrincipalBytes;

      if (isSharedFile) {
        console.log("Getting owner principal for shared file");
        try {
          // Get the file owner's principal directly from our endpoint
          const ownerPrincipalResponse =
            await this.actor.get_file_owner_principal(fileId);

          if (!ownerPrincipalResponse || "Err" in ownerPrincipalResponse) {
            throw new Error("Error getting file owner principal");
          }

          principalToUse = new Uint8Array(ownerPrincipalResponse.Ok);
          console.log(
            "Using owner's principal for shared file:",
            principalToUse,
          );
        } catch (e) {
          console.error(
            "Failed to get owner principal, falling back to user principal",
            e,
          );
        }
      } else {
        console.log("Using current user's principal (file owner)");
      }

      // Get encrypted key from the backend
      const privateKeyResponse = await this.actor.vetkd_encrypted_key(
        transportSecretKey.public_key(),
        [fileId],
      );
      console.log("privateKeyResponse: ", privateKeyResponse);

      if (!privateKeyResponse || "Err" in privateKeyResponse) {
        throw new Error(
          "Error getting encrypted key: " +
            ("Err" in privateKeyResponse
              ? privateKeyResponse.Err
              : "empty response"),
        );
      }
      const encryptedKey = privateKeyResponse.Ok as Uint8Array;
      console.log("encryptedKey: ", encryptedKey);

      // Decrypt the key with the transport secret
      const key = transportSecretKey.decrypt(
        encryptedKey,
        publicKey,
        principalToUse, // Use owner's principal if provided
      );
      console.log("key: ", key);

      // Deserialize and decrypt the data
      const ibeCiphertext = vetkd.IBECiphertext.deserialize(encryptedData);
      return ibeCiphertext.decrypt(key);
    } catch (error) {
      console.error("Decryption error:", error);
      throw error;
    }

    // Helper method to compare Uint8Arrays
    function equalUint8Arrays(a: Uint8Array, b: Uint8Array): boolean {
      if (a.length !== b.length) return false;
      for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return false;
      }
      return true;
    }
  }
}
