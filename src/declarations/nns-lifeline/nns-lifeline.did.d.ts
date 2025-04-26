import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CanisterSettings {
  'freezing_threshold' : [] | [bigint],
  'wasm_memory_threshold' : [] | [bigint],
  'controllers' : [] | [Array<Principal>],
  'reserved_cycles_limit' : [] | [bigint],
  'log_visibility' : [] | [LogVisibility],
  'wasm_memory_limit' : [] | [bigint],
  'memory_allocation' : [] | [bigint],
  'compute_allocation' : [] | [bigint],
}
export interface HardResetRootToVersionPayload {
  'wasm_module' : Uint8Array | number[],
  'init_arg' : Uint8Array | number[],
}
export type LogVisibility = { 'controllers' : null } |
  { 'public' : null };
export interface UpgradeRootProposalPayload {
  'wasm_module' : Uint8Array | number[],
  'module_arg' : Uint8Array | number[],
  'stop_upgrade_start' : boolean,
}
export interface _SERVICE {
  'hard_reset_root_to_version' : ActorMethod<
    [HardResetRootToVersionPayload],
    undefined
  >,
  'upgrade_root' : ActorMethod<[UpgradeRootProposalPayload], undefined>,
  'upgrade_root_settings' : ActorMethod<[CanisterSettings], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
