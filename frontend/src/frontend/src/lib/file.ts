import { default as crypto } from "./crypto";
import { VetKeyService } from "./vetkeys/encrypt";

/**
 * A file that is backed by the Canister backend service.
 */
// class File {
//   /**
//    * Initializes a file from unencrypted data.
//    *
//    * @param {String} name
//    * @param {ArrayBuffer} contents
//    *
//    * @returns {File}
//    */
//   static fromUnencrypted(name, contents) {
//     return new File(name, contents);
//   }

//   /**
//    * Initializes a file from encrypted data.
//    *
//    * @param {String} name
//    * @param {ArrayBuffer} encryptedBytes
//    * @param {ArrayBuffer} encryptedDocumentKey
//    *
//    * @returns {Promise<File>}
//    */
//   static async fromEncrypted(name, encryptedBytes, encryptedDocumentKey) {
//     // Decrypt the file's key using the user's key.
//     const documentKey = await crypto.decryptForUser(encryptedDocumentKey);

//     // Decrypt the document.
//     const contents = await crypto.decryptFile(encryptedBytes, documentKey);

//     return new File(name, contents);
//   }

//   /**
//    * @param {String} name
//    * @param {ArrayBuffer} contents
//    */
//   constructor(name, contents) {
//     this.name = name;
//     this.contents = contents;
//     this.documentKey = null;
//   }

//   /**
//    * @returns {Promise<ArrayBuffer>} the encrypted bytes corresponding to this file.
//    */
//   async encrypt() {
//     const documentKey = await this._getFileKey();
//     return crypto.encryptFile(this.contents, documentKey);
//   }

//   /**
//    * @param {ArrayBuffer} userPublicKey
//    * @returns {Promise<ArrayBuffer>} the file's key encryped with the user's public key.
//    */
//   async getEncryptedFileKey(userPublicKey) {
//     const key = await this._getFileKey();
//     // Encrypt the exported key with the user's public key.
//     return await crypto.encryptForUser(key, userPublicKey);
//   }

//   /**
//    * @returns {Promise<CryptoKey>} the file's key. A key is generated if it doesn't exist.
//    */
//   async _getFileKey() {
//     if (this.documentKey == null) {
//       this.documentKey = await crypto.generateFileKey();
//     }
//     return this.documentKey;
//   }
// }

class CustomFile {
  static async fromEncrypted(
    name: string,
    encryptedBytes: string,
    fileId: bigint,
    vetKeyService: VetKeyService,
  ) {
    const contents = await vetKeyService.decrypt(fileId, encryptedBytes);
    return new File(name, contents);
  }

  constructor(
    public name: string,
    public contents: ArrayBuffer,
  ) {}

  async encrypt(fileId: bigint, vetKeyService: VetKeyService): Promise<string> {
    return await vetKeyService.encrypt(fileId, this.contents);
  }
}

export default CustomFile;
