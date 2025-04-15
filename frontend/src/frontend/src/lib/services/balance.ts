import { LedgerCanister, AccountIdentifier } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';
import type { AuthClient } from '@dfinity/auth-client';
import { writable } from 'svelte/store';
import { createAgent } from "@dfinity/utils";
import { Identity } from '@dfinity/agent';

const ICP_LEDGER_CANISTER_ID = Principal.fromText('ryjl3-tyaaa-aaaaa-aaaba-cai');
const host = import.meta.env.VITE_HOST || (import.meta.env.DEV ? 'http://127.0.0.1:8080' : 'https://icp-api.io');

export type BalanceState = {
  balance: bigint | null;
  loading: boolean;
  error: string | null;
};

export class BalanceService {
  public store = writable<BalanceState>({
    balance: null,
    loading: false,
    error: null,
  });

  private ledgerCanister: LedgerCanister | null = null;
  private principal: Principal | null = null;

  private isInitialized = false; // Flag to track initialization status

  constructor(private authClient: AuthClient) {
    // Don't call initialize directly, let it be called explicitly or lazily
    console.log("BalanceService: Constructor called.");
  }

  // Make initialize public and ensure it runs only once or when needed
  async initialize(identity?: Identity, hostOverride?: string): Promise<boolean> {
    if (this.isInitialized) {
      console.log("BalanceService: Already initialized.");
      return true;
    }

    const effectiveIdentity = identity ?? this.authClient?.getIdentity();
    const effectiveHost = hostOverride ?? host;

    if (!effectiveIdentity) {
        console.error("BalanceService: Cannot initialize without identity.");
        return false;
    }

    console.log("BalanceService: Initializing...");
    try {
      console.log("BalanceService: Creating agent with host:", effectiveHost);
      const agent = await createAgent({
        identity: effectiveIdentity,
        host: effectiveHost,
      });
      console.log("BalanceService: Agent created:", agent ? 'Success' : 'Failed');

      this.principal = effectiveIdentity.getPrincipal();
      console.log("BalanceService: Principal obtained:", this.principal?.toText());

      console.log("BalanceService: Creating LedgerCanister with canisterId:", ICP_LEDGER_CANISTER_ID.toText());
      this.ledgerCanister = LedgerCanister.create({
        agent,
        canisterId: ICP_LEDGER_CANISTER_ID,
      });
      console.log("BalanceService: LedgerCanister created:", this.ledgerCanister ? 'Success' : 'Failed');

      this.isInitialized = !!(this.ledgerCanister && this.principal);
      console.log("BalanceService: Initialization complete. Status:", this.isInitialized);
      return this.isInitialized;

    } catch (error) {
        console.error("BalanceService: Initialization failed:", error);
        this.isInitialized = false;
        this.ledgerCanister = null;
        this.principal = null;
        return false;
    }
  }

  async fetchBalance(certified = false): Promise<void> {
    // Ensure initialization before fetching
    if (!this.isInitialized) {
        console.log("BalanceService: Attempting lazy initialization before fetch...");
        const initialized = await this.initialize();
        if (!initialized) {
            this.store.set({ balance: null, loading: false, error: 'Initialization failed. Cannot fetch balance.' });
            console.error('BalanceService: Initialization failed. Cannot fetch balance.');
            return;
        }
    }

    // Double check after attempting initialization
    if (!this.ledgerCanister || !this.principal) {
      console.log("BalanceService: Fetch failed. Ledger: ", this.ledgerCanister, "Principal: ", this.principal);
      this.store.set({ balance: null, loading: false, error: 'Balance service state invalid after initialization attempt.' });
      console.error('BalanceService: State invalid after initialization attempt.');
      return;
    }

    console.log("BalanceService: Fetching balance for principal:", this.principal.toText());

    this.store.update(s => ({ ...s, loading: true, error: null }));

    try {
      const accountIdentifier = AccountIdentifier.fromPrincipal({ principal: this.principal });
      const userBalance = await this.ledgerCanister.accountBalance({
        accountIdentifier,
        certified,
      });
      this.store.set({ balance: userBalance, loading: false, error: null });
    } catch (error) {
      console.error('Error fetching balance:', error);
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
      this.store.set({ balance: null, loading: false, error: `Failed to fetch balance: ${errorMessage}` });
    }
  }

  // Call this method if the authClient instance changes or needs refreshing
  async refreshServiceState(newAuthClient?: AuthClient) {
    console.log("BalanceService: Refreshing service state...");
    if (newAuthClient) {
        this.authClient = newAuthClient;
    }
    this.isInitialized = false; // Force re-initialization on next call or fetch
    // Optionally trigger re-initialization immediately:
    // await this.initialize();
    // Optionally re-fetch balance immediately after refresh:
    // await this.fetchBalance();
  }

  reset() {
    console.log("BalanceService: Resetting service state.");
    this.store.set({ balance: null, loading: false, error: null });
    this.ledgerCanister = null;
    this.principal = null;
    this.isInitialized = false; // Reset initialization flag
  }
}
