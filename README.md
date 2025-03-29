# Canister Cloud

Canister Cloud is a secure, scalable, and fully on-chain document storage and management dApp. Built on the **Internet Computer (ICP)** by **Dfinity**, it leverages blockchain technology to provide a decentralized, tamper-proof solution for document management. 

## Features

- **Drag-and-Drop File Upload** – Easily upload documents for streamlined management.
- **Enhanced Document Requests and Sharing** – Request multiple documents in a single workflow and create workflow templates.
- **VETKey-Backed Storage Security** – Strengthened file storage with VETKey-based encryption.
- **File Management Tools** – Rename and delete files for better organization.

## Technology Stack

Canister Cloud is developed using:
- **Rust**
- **Svelte**
- **TailwindCSS**
- **Internet Identity** – Secure, decentralized authentication for user access.
- **VETKeys** – Advanced encryption mechanism for securing document storage and access.

## Development

To run the dApp locally, start the Internet Computer backend:

```sh
# Start the local DFX environment
dfx start --clean
```

In another terminal, install dependencies and deploy the canisters:

```sh
# Install frontend dependencies
npm install -g pnpm
pnpm install

# Deploy the canisters
dfx deploy
```

## Local Frontend Development

After deploying Internet Identity and the backend canister locally, you can start the dApp frontend:

```sh
pnpm --filter frontend run dev
```

## Resources

- **Dapp:** [www.canister.co](https://www.canister.co)
- **GitHub Repo:** [https://github.com/jakepeg/canister_app](https://github.com/jakepeg/canister_app)
- **X/Twitter:** [https://x.com/CanisterCloud](https://x.com/CanisterCloud)
- **LinkedIn:** [https://www.linkedin.com/company/canister-cloud/](https://www.linkedin.com/company/canister-cloud/)
