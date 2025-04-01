import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CanisterInfo { 'id' : Principal, 'name' : string }
export type GetUserCanistersResponse = { 'Ok' : Array<CanisterInfo> } |
  { 'NotAuthenticated' : null };
export type RegisterCanisterResponse = { 'Ok' : null } |
  { 'AlreadyRegistered' : null } |
  { 'NotAuthorized' : null } |
  { 'VerificationFailed' : string } |
  { 'InternalError' : string };
export type VetkdEncryptedKeyResponse = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type VetkdPublicKeyResponse = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type download_file_response = { 'found_file' : found_file } |
  { 'permission_error' : null } |
  { 'not_uploaded_file' : null } |
  { 'not_found_file' : null };
export interface file {
  'contents' : [] | [Uint8Array | number[]],
  'metadata' : file_metadata,
}
export type file_id = bigint;
export interface file_info {
  'alias' : string,
  'file_name' : string,
  'file_id' : file_id,
}
export interface file_metadata {
  'file_status' : file_status,
  'group_alias' : [] | [string],
  'file_name' : string,
  'shared_with' : Array<user>,
  'group_name' : string,
  'file_id' : file_id,
}
export type file_status = { 'partially_uploaded' : null } |
  { 'pending' : { 'alias' : string, 'requested_at' : bigint } } |
  { 'uploaded' : { 'uploaded_at' : bigint } };
export interface found_file {
  'contents' : Uint8Array | number[],
  'file_type' : string,
  'num_chunks' : bigint,
}
export type get_alias_info_response = {
    'Ok' : { 'user' : user, 'file_name' : string, 'file_id' : file_id }
  } |
  { 'Err' : { 'not_found' : null } };
export type get_users_response = { 'permission_error' : null } |
  { 'users' : Array<user> };
export interface group_info {
  'files' : Array<file_info>,
  'requester' : user,
  'group_id' : bigint,
  'group_name' : string,
}
export interface multi_request_input {
  'file_names' : Array<string>,
  'save_as_template' : boolean,
  'group_name' : string,
}
export interface multi_request_response {
  'group_alias' : string,
  'group_id' : bigint,
}
export interface public_request_group {
  'files' : Array<file_metadata>,
  'name' : string,
  'created_at' : bigint,
  'group_id' : bigint,
}
export type set_user_response = { 'ok' : null } |
  { 'username_exists' : null };
export type share_file_response = { 'ok' : null } |
  { 'permission_error' : null };
export interface template { 'file_names' : Array<string>, 'name' : string }
export type template_response = { 'Ok' : template } |
  { 'Err' : { 'not_found' : null } };
export interface upload_file_atomic_request {
  'content' : Uint8Array | number[],
  'name' : string,
  'file_type' : string,
  'num_chunks' : bigint,
}
export interface upload_file_continue_request {
  'contents' : Uint8Array | number[],
  'chunk_id' : bigint,
  'file_id' : file_id,
}
export type upload_file_error = { 'not_requested' : null } |
  { 'already_uploaded' : null };
export interface upload_file_request {
  'file_type' : string,
  'num_chunks' : bigint,
  'file_content' : Uint8Array | number[],
  'file_id' : file_id,
}
export type upload_file_response = { 'Ok' : null } |
  { 'Err' : upload_file_error };
export interface user {
  'username' : string,
  'public_key' : Uint8Array | number[],
  'ic_principal' : Principal,
}
export type who_am_i_response = { 'known_user' : { 'username' : string } } |
  { 'unknown_user' : null };
export interface _SERVICE {
  'delete_file' : ActorMethod<[file_id], share_file_response>,
  'delete_template' : ActorMethod<[string], undefined>,
  'download_file' : ActorMethod<[file_id, bigint], download_file_response>,
  'get_alias_info' : ActorMethod<[string], get_alias_info_response>,
  'get_file_owner_principal' : ActorMethod<
    [bigint],
    { 'Ok' : Uint8Array | number[] } |
      { 'Err' : string }
  >,
  'get_group_by_alias' : ActorMethod<
    [string],
    { 'Ok' : group_info } |
      { 'Err' : { 'not_found' : null } }
  >,
  'get_request_groups' : ActorMethod<[], Array<public_request_group>>,
  'get_requests' : ActorMethod<[], Array<file_metadata>>,
  'get_shared_files' : ActorMethod<[], Array<file_metadata>>,
  'get_template' : ActorMethod<[string], template_response>,
  'get_template_names' : ActorMethod<[], Array<string>>,
  'get_user_canisters' : ActorMethod<[], GetUserCanistersResponse>,
  'get_user_templates' : ActorMethod<[], Array<template>>,
  'get_users' : ActorMethod<[], get_users_response>,
  'hello_world' : ActorMethod<[], string>,
  'multi_request' : ActorMethod<[multi_request_input], multi_request_response>,
  'register_canister' : ActorMethod<
    [Principal, string],
    RegisterCanisterResponse
  >,
  'rename_file' : ActorMethod<[file_id, string], share_file_response>,
  'request_file' : ActorMethod<[string], string>,
  'revoke_share' : ActorMethod<[Principal, file_id], share_file_response>,
  'set_user' : ActorMethod<[string, Uint8Array | number[]], set_user_response>,
  'share_file' : ActorMethod<[Principal, file_id], share_file_response>,
  'share_file_with_users' : ActorMethod<[Array<Principal>, file_id], undefined>,
  'upload_file' : ActorMethod<[upload_file_request], upload_file_response>,
  'upload_file_atomic' : ActorMethod<[upload_file_atomic_request], file_id>,
  'upload_file_continue' : ActorMethod<
    [upload_file_continue_request],
    undefined
  >,
  'username_exists' : ActorMethod<[string], boolean>,
  'vetkd_encrypted_key' : ActorMethod<
    [Uint8Array | number[], [] | [bigint]],
    VetkdEncryptedKeyResponse
  >,
  'vetkd_public_key' : ActorMethod<[], VetkdPublicKeyResponse>,
  'who_am_i' : ActorMethod<[], who_am_i_response>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
