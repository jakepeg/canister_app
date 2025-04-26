import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AddCanisterRequest {
  'arg' : Uint8Array | number[],
  'initial_cycles' : bigint,
  'wasm_module' : Uint8Array | number[],
  'name' : string,
  'memory_allocation' : [] | [bigint],
  'compute_allocation' : [] | [bigint],
}
export type CanisterAction = { 'Start' : null } |
  { 'Stop' : null };
export interface CanisterIdRecord { 'canister_id' : Principal }
export type CanisterInstallMode = { 'reinstall' : null } |
  { 'upgrade' : null } |
  { 'install' : null };
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
export type CanisterStatusLogVisibility = { 'controllers' : null } |
  { 'public' : null } |
  { 'allowed_viewers' : Array<Principal> };
export interface CanisterStatusResult {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'query_stats' : [] | [QueryStats],
  'idle_cycles_burned_per_day' : [] | [bigint],
  'module_hash' : [] | [Uint8Array | number[]],
  'reserved_cycles' : [] | [bigint],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export interface ChangeCanisterControllersError {
  'code' : [] | [number],
  'description' : string,
}
export interface ChangeCanisterControllersRequest {
  'target_canister_id' : Principal,
  'new_controllers' : Array<Principal>,
}
export interface ChangeCanisterControllersResponse {
  'change_canister_controllers_result' : ChangeCanisterControllersResult,
}
export type ChangeCanisterControllersResult = { 'Ok' : null } |
  { 'Err' : ChangeCanisterControllersError };
export interface ChangeCanisterRequest {
  'arg' : Uint8Array | number[],
  'wasm_module' : Uint8Array | number[],
  'stop_before_installing' : boolean,
  'mode' : CanisterInstallMode,
  'canister_id' : Principal,
  'chunked_canister_wasm' : [] | [ChunkedCanisterWasm],
  'memory_allocation' : [] | [bigint],
  'compute_allocation' : [] | [bigint],
}
export interface ChunkedCanisterWasm {
  'wasm_module_hash' : Uint8Array | number[],
  'chunk_hashes_list' : Array<Uint8Array | number[]>,
  'store_canister_id' : Principal,
}
export interface DefiniteCanisterSettings {
  'freezing_threshold' : [] | [bigint],
  'wasm_memory_threshold' : [] | [bigint],
  'controllers' : Array<Principal>,
  'reserved_cycles_limit' : [] | [bigint],
  'log_visibility' : [] | [CanisterStatusLogVisibility],
  'wasm_memory_limit' : [] | [bigint],
  'memory_allocation' : [] | [bigint],
  'compute_allocation' : [] | [bigint],
}
export type LogVisibility = { 'controllers' : null } |
  { 'public' : null };
export interface QueryStats {
  'response_payload_bytes_total' : [] | [bigint],
  'num_instructions_total' : [] | [bigint],
  'num_calls_total' : [] | [bigint],
  'request_payload_bytes_total' : [] | [bigint],
}
export interface StopOrStartCanisterRequest {
  'action' : CanisterAction,
  'canister_id' : Principal,
}
export interface UpdateCanisterSettingsError {
  'code' : [] | [number],
  'description' : string,
}
export interface UpdateCanisterSettingsRequest {
  'canister_id' : Principal,
  'settings' : CanisterSettings,
}
export type UpdateCanisterSettingsResponse = { 'Ok' : null } |
  { 'Err' : UpdateCanisterSettingsError };
export interface _SERVICE {
  'add_nns_canister' : ActorMethod<[AddCanisterRequest], undefined>,
  'canister_status' : ActorMethod<[CanisterIdRecord], CanisterStatusResult>,
  'change_canister_controllers' : ActorMethod<
    [ChangeCanisterControllersRequest],
    ChangeCanisterControllersResponse
  >,
  'change_nns_canister' : ActorMethod<[ChangeCanisterRequest], undefined>,
  'get_build_metadata' : ActorMethod<[], string>,
  'stop_or_start_nns_canister' : ActorMethod<
    [StopOrStartCanisterRequest],
    undefined
  >,
  'update_canister_settings' : ActorMethod<
    [UpdateCanisterSettingsRequest],
    UpdateCanisterSettingsResponse
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
