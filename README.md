# Canister Cloud

You can upload documents and authorize people to access them with a few clicks. Access to shared documents can expire or be explicitly revoked. In addition, you can ask other people to upload documents for you by simply sharing a link (no login required). Documents are transmitted and stored in encrypted form. The dapp can be used with any standard web browser, no plugins or extensions are needed and no passwords need to be remembered.

## Development

To run the dapp locally, run the following in one terminal window:

```
dfx start --clean
```

And in another terminal"

```
# Install needed frontend dependencies.
npm install -g pnpm
pnpm install

# Deploy the canisters.
dfx deploy
dfx deps pull
dfx deps deploy
```

In your browser you can now go to <canister_id>.localhost:8000 to access the frontend.

If you want to contribute, see our [CONTRIBUTING](.github/CONTRIBUTING.md) document to get started.

## Local frontend development

After deploying locally both Internet Identity and the backend canister, you can run the dapp frontend and the home page development server.

### Frontend project of the dapp

```
pnpm --filter frontend run dev
```

### Home page

```
pnpm --filter landing-page run dev
```
