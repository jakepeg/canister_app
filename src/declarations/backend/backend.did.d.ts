import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CanisterInfo { 'id' : Principal, 'name' : string }
export type DeleteCanisterResponse = { 'Ok' : null } |
  { 'CanisterNotFound' : null } |
  { 'DeletionFailed' : string } |
  { 'NotAuthorized' : null } |
  { 'InternalError' : string };
export type GetUserCanistersResponse = { 'Ok' : Array<CanisterInfo> } |
  { 'NotAuthenticated' : null };
export type RegisterCanisterResponse = { 'Ok' : null } |
  { 'AlreadyRegistered' : null } |
  { 'NotAuthorized' : null } |
  { 'VerificationFailed' : string } |
  { 'InternalError' : string };
export type RenameCanisterResponse = { 'Ok' : null } |
  { 'CanisterNotFound' : null } |
  { 'NotAuthorized' : null } |
  { 'InternalError' : string };
export type VetkdEncryptedKeyResponse = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type VetkdPublicKeyResponse = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export interface alias_info_response {
  'user' : user,
  'file_name' : string,
  'item_id' : item_id,
}
export type download_file_chunk_response = { 'permission_error' : null } |
  { 'not_a_file' : null } |
  { 'chunk_not_found' : null } |
  { 'found_file_chunk' : found_file_chunk } |
  { 'not_uploaded_file' : null } |
  { 'not_found_item' : null };
export interface file_info_for_upload {
  'alias' : string,
  'file_name' : string,
  'item_id' : item_id,
}
export type file_status_legacy = { 'partially_uploaded' : null } |
  { 'pending' : { 'alias' : string, 'requested_at' : bigint } } |
  { 'uploaded' : { 'uploaded_at' : bigint } };
export interface found_file_chunk {
  'contents' : Uint8Array | number[],
  'file_type' : string,
  'num_chunks' : bigint,
}
export type get_alias_info_error = { 'not_found' : null };
export interface group_info_response {
  'files' : Array<file_info_for_upload>,
  'requester' : user,
  'group_id' : item_id,
  'group_name' : string,
}
export type item_id = bigint;
export type item_operation_response = { 'Ok' : null } |
  { 'Err' : string };
export type item_operation_response_detailed = { 'Ok' : null } |
  { 'AlreadyUploaded' : null } |
  { 'FolderNotEmpty' : null } |
  { 'ItemNotFound' : null } |
  { 'NotAFile' : null } |
  { 'NotAFolder' : null } |
  { 'NotRequested' : null } |
  { 'PermissionError' : null } |
  { 'PendingError' : null };
export type item_type = { 'Folder' : null } |
  { 'File' : null };
export interface multi_request_input {
  'file_names' : Array<string>,
  'save_as_template' : boolean,
  'group_name' : string,
}
export interface multi_request_response {
  'group_alias' : string,
  'group_id' : item_id,
}
export interface public_file_metadata_legacy {
  'file_status' : file_status_legacy,
  'group_alias' : [] | [string],
  'file_name' : string,
  'shared_with' : Array<user>,
  'group_name' : string,
  'file_id' : item_id,
}
export interface public_item_metadata {
  'id' : item_id,
  'modified_at' : bigint,
  'name' : string,
  'size' : [] | [bigint],
  'item_type' : item_type,
  'parent_id' : [] | [item_id],
}
export interface public_request_group {
  'files' : Array<public_item_metadata>,
  'name' : string,
  'created_at' : bigint,
  'group_id' : item_id,
}
export type set_user_response = { 'ok' : null } |
  { 'username_exists' : null };
export interface template { 'file_names' : Array<string>, 'name' : string }
export type template_response = { 'Ok' : template } |
  { 'Err' : { 'not_found' : null } };
export interface upload_chunk_continue_request {
  'contents' : Uint8Array | number[],
  'chunk_id' : bigint,
  'item_id' : item_id,
}
export interface upload_file_atomic_request_new {
  'content' : Uint8Array | number[],
  'name' : string,
  'file_type' : string,
  'parent_id' : [] | [item_id],
  'num_chunks' : bigint,
}
export type upload_file_error_legacy = { 'not_requested' : null } |
  { 'already_uploaded' : null };
export type upload_file_response_legacy = { 'Ok' : null } |
  { 'Err' : upload_file_error_legacy };
export interface upload_file_to_item_request {
  'file_type' : string,
  'num_chunks' : bigint,
  'file_content' : Uint8Array | number[],
  'item_id' : item_id,
}
export interface user {
  'username' : string,
  'public_key' : Uint8Array | number[],
  'ic_principal' : Principal,
}
export type who_am_i_response = { 'known_user' : user } |
  { 'unknown_user' : null };
export interface _SERVICE {
  'create_folder' : ActorMethod<
    [string, [] | [item_id]],
    { 'Ok' : public_item_metadata } |
      { 'Err' : string }
  >,
  'delete_item' : ActorMethod<[item_id], item_operation_response>,
  'delete_template' : ActorMethod<[string], item_operation_response>,
  'download_file_chunk' : ActorMethod<
    [item_id, bigint],
    download_file_chunk_response
  >,
  'get_alias_info' : ActorMethod<
    [string],
    { 'Ok' : alias_info_response } |
      { 'Err' : get_alias_info_error }
  >,
  'get_group_by_alias' : ActorMethod<
    [string],
    { 'Ok' : group_info_response } |
      { 'Err' : get_alias_info_error }
  >,
  'get_item_metadata_by_id' : ActorMethod<
    [item_id],
    { 'Ok' : public_item_metadata } |
      { 'Err' : string }
  >,
  'get_item_sharers' : ActorMethod<
    [item_id],
    { 'Ok' : Array<user> } |
      { 'Err' : string }
  >,
  'get_items_shared_with_me' : ActorMethod<[], Array<public_item_metadata>>,
  'get_my_pending_requests' : ActorMethod<[], Array<public_item_metadata>>,
  'get_request_groups' : ActorMethod<[], Array<public_request_group>>,
  'get_template' : ActorMethod<[string], template_response>,
  'get_template_names' : ActorMethod<[], Array<string>>,
  'get_user_canisters' : ActorMethod<[], GetUserCanistersResponse>,
  'get_user_templates' : ActorMethod<[], Array<template>>,
  'get_users' : ActorMethod<[], { 'Ok' : Array<user> } | { 'Err' : string }>,
  'hello_world' : ActorMethod<[], string>,
  'list_folder_contents' : ActorMethod<
    [[] | [item_id]],
    { 'Ok' : Array<public_item_metadata> } |
      { 'Err' : string }
  >,
  'move_item' : ActorMethod<[item_id, [] | [item_id]], item_operation_response>,
  'multi_request' : ActorMethod<
    [multi_request_input],
    { 'Ok' : multi_request_response } |
      { 'Err' : string }
  >,
  'register_canister' : ActorMethod<
    [Principal, string],
    RegisterCanisterResponse
  >,
  'rename_canister' : ActorMethod<[Principal, string], RenameCanisterResponse>,
  'rename_item' : ActorMethod<[item_id, string], item_operation_response>,
  'request_file' : ActorMethod<
    [string, [] | [item_id]],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'revoke_item_share' : ActorMethod<
    [Principal, item_id],
    item_operation_response
  >,
  'set_user' : ActorMethod<[string, Uint8Array | number[]], set_user_response>,
  'share_item' : ActorMethod<[Principal, item_id], item_operation_response>,
  'unregister_canister' : ActorMethod<[Principal], DeleteCanisterResponse>,
  'upload_chunk_continue' : ActorMethod<
    [upload_chunk_continue_request],
    item_operation_response_detailed
  >,
  'upload_file_atomic' : ActorMethod<
    [upload_file_atomic_request_new],
    { 'Ok' : item_id } |
      { 'Err' : string }
  >,
  'upload_file_to_item' : ActorMethod<
    [upload_file_to_item_request],
    item_operation_response_detailed
  >,
  'username_exists' : ActorMethod<[string], boolean>,
  'vetkd_encrypted_key' : ActorMethod<
    [Uint8Array | number[], [] | [item_id]],
    VetkdEncryptedKeyResponse
  >,
  'vetkd_public_key' : ActorMethod<[], VetkdPublicKeyResponse>,
  'who_am_i' : ActorMethod<[], who_am_i_response>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
