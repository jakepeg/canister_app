// npm run dev = local
// npm run build = local
// dfx deploy = local

import { execSync } from "child_process";

// dfx deploy --network ic = ic
export const network = process.env.DFX_NETWORK ?? "local";
export const host =
  process.env.DFX_NETWORK_HOST ??
  (network === "local" ? "http://127.0.0.1:8080" : "https://icp0.io");
// Note regarding iiUrl:  On Safari, localhost subdomains are not supported.  If developing with Safari, please use
// II_URL=http://127.0.0.1:8000/?canisterId=rdmx6-jaaaa-aaaaa-aaadq-cai or similar.
export const iiUrl =
  process.env.II_URL ??
  (network === "local"
    ? "http://be2us-64aaa-aaaaa-qaabq-cai.localhost:8080/"
    : "https://identity.ic0.app");

export const readCanisterIds = ({
  prefix,
}: {
  prefix?: string;
}): Record<string, string> => {
  try {
    let canisters = [
      "frontend",
      "backend",
      "internet_identity",
      "vetkd_system_api",
      "nns-cycles-minting",
      "nns-genesis-token",
      "nns-governance",
      "nns-ledger",
      "nns-lifeline",
      "nns-registry",
      "nns-root",
      "nns-sns-wasm",
    ];
    return canisters.reduce(
      (acc, canisterName) => ({
        ...acc,
        // Replace hyphens with underscores for valid env var names
        [`${prefix ?? ""}${canisterName.replace(/-/g, "_").toUpperCase()}_CANISTER_ID`]: execSync(
          `dfx canister id --network ${network} ${canisterName}`, // Still use original name for dfx command
        )
          .toString()
          .trim(),
      }),
      {},
    );
  } catch (e) {
    throw Error(`Could not get canister IDs: ${e}`);
  }
};
