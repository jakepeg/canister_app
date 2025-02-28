Thank you both so much @Severin and @kristofer,

At least now i understand the candid ui not working is not a deal breaker for working with vetkey's locally. 

Right now I'm in the process of refactoring the project from using standard double layer encryption to using vetkeys which requires a decent amount of architectural changes. 

Please ignore the part below because it's just me trying to organize my thoughts and trying to properly architect the change to vetkeys.

# Current flow:
**A. Initial Setup:**
- Each user has a RSA keypair stored in their browser

**B. File Upload Process:**
1. When uploading a file:
   - Generate a random AES key for the file
   - Encrypt file with AES key
   - Encrypt AES key with recipient's public RSA key
   - Upload both encrypted file and encrypted AES key

**C. File Decryption:**

1. When downloading a file:
   - Download encrypted file and encrypted AES key
   - Decrypt AES key using private RSA key
   - Use decrypted AES key to decrypt file contents

# Refactored flow:
**A. Initial Setup:**
- Replace `keyStorage.ts` with vetkeys storage system
- Create new `VetKeyService` class in `lib/vetkeys/encrypt.ts` that will be our main interface with the vetkeys package

**B. File Upload Process:**
1. In `lib/components/Upload/Upload.svelte`:
   - User selects file
   - System gets note ID from backend
   - File is encrypted using vetkeys' `encryptWithNoteKey`
   - Encrypted file is uploaded in chunks

3. Core encryption logic in `VetKeyService`:
   - Instead of generating AES keys, use IBE-based encryption
   - Each file gets a unique note ID
   - Encryption is tied to the owner's identity

**C. File Decryption:**
1. In `DecryptService`:
   - Download encrypted file
   - Use `decryptWithNoteKey` to decrypt content
   - Permissions are automatically handled by the vetkeys system

Current issue is I'm trying to use the already built @shipstone's vetkey package and due to it having tools prebuilt for file/note encryption.

I thought it would be fitting but unfortunately I am getting more confused, could be cause I'm sick right now but also just not sure how to properly generate the noteId required for their encryption methods and how they need to relate to the actual documents (e.g. do they need to be derived in some way through hashing?). 

Thinking of just doing it the same way it was done in @kristofer's repo with vetkd.IBECiphertext.encrypt method which might just be simpler.

Btw, if anyone knows how to get in touch with the @shipstone  team I would really appreciate it

Goal
1. make a minimum upload and display svelte component for files with vetkey encryption
2. all in one file to avoid complexity in debugging 
3. important part is to make vetkey encryption and decryption work
Goddamn there are some questions and I didn't notice, sorry for that, if anyone needs any help please reach out to me in my socials either through BlueSky or any of my other socials
Discord: https://discordapp.com/users/ahegao_burger
Telegram: https://t.me/Nth_Typonomy
