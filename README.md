# Canister Cloud

Canister Cloud is a secure, scalable, and fully on-chain document storage and management dApp designed for Human Resources (HR) departments. It streamlines HR workflows, particularly for new hire onboarding, by simplifying the collection of signed contracts, work permits, identification documents, and other essential paperwork.

## Features

- **Drag-and-Drop File Upload** – Easily upload documents for streamlined management.
- **Enhanced Document Requests and Sharing** – Request multiple documents in a single workflow and create workflow templates.
- **VETKey-Backed Storage Security** – Strengthened file storage with VETKey-based encryption.
- **File Management Tools** – Rename and delete files for better organization.

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
