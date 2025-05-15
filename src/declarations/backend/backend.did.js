export const idlFactory = ({ IDL }) => {
  const item_id = IDL.Nat64;
  const item_type = IDL.Variant({ 'Folder' : IDL.Null, 'File' : IDL.Null });
  const public_item_metadata = IDL.Record({
    'id' : item_id,
    'modified_at' : IDL.Nat64,
    'name' : IDL.Text,
    'size' : IDL.Opt(IDL.Nat64),
    'item_type' : item_type,
    'parent_id' : IDL.Opt(item_id),
  });
  const item_operation_response = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  const found_file_chunk = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
  });
  const download_file_chunk_response = IDL.Variant({
    'permission_error' : IDL.Null,
    'not_a_file' : IDL.Null,
    'chunk_not_found' : IDL.Null,
    'found_file_chunk' : found_file_chunk,
    'not_uploaded_file' : IDL.Null,
    'not_found_item' : IDL.Null,
  });
  const user = IDL.Record({
    'username' : IDL.Text,
    'public_key' : IDL.Vec(IDL.Nat8),
    'ic_principal' : IDL.Principal,
  });
  const alias_info_response = IDL.Record({
    'user' : user,
    'file_name' : IDL.Text,
    'item_id' : item_id,
  });
  const get_alias_info_error = IDL.Variant({ 'not_found' : IDL.Null });
  const file_info_for_upload = IDL.Record({
    'alias' : IDL.Text,
    'file_name' : IDL.Text,
    'item_id' : item_id,
  });
  const group_info_response = IDL.Record({
    'files' : IDL.Vec(file_info_for_upload),
    'requester' : user,
    'group_id' : item_id,
    'group_name' : IDL.Text,
  });
  const public_request_group = IDL.Record({
    'files' : IDL.Vec(public_item_metadata),
    'name' : IDL.Text,
    'created_at' : IDL.Nat64,
    'group_id' : item_id,
  });
  const template = IDL.Record({
    'file_names' : IDL.Vec(IDL.Text),
    'name' : IDL.Text,
  });
  const template_response = IDL.Variant({
    'Ok' : template,
    'Err' : IDL.Variant({ 'not_found' : IDL.Null }),
  });
  const CanisterInfo = IDL.Record({ 'id' : IDL.Principal, 'name' : IDL.Text });
  const GetUserCanistersResponse = IDL.Variant({
    'Ok' : IDL.Vec(CanisterInfo),
    'NotAuthenticated' : IDL.Null,
  });
  const multi_request_input = IDL.Record({
    'file_names' : IDL.Vec(IDL.Text),
    'save_as_template' : IDL.Bool,
    'group_name' : IDL.Text,
  });
  const multi_request_response = IDL.Record({
    'group_alias' : IDL.Text,
    'group_id' : item_id,
  });
  const RegisterCanisterResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyRegistered' : IDL.Null,
    'NotAuthorized' : IDL.Null,
    'VerificationFailed' : IDL.Text,
    'InternalError' : IDL.Text,
  });
  const RenameCanisterResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'CanisterNotFound' : IDL.Null,
    'NotAuthorized' : IDL.Null,
    'InternalError' : IDL.Text,
  });
  const set_user_response = IDL.Variant({
    'ok' : IDL.Null,
    'username_exists' : IDL.Null,
  });
  const DeleteCanisterResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'CanisterNotFound' : IDL.Null,
    'DeletionFailed' : IDL.Text,
    'NotAuthorized' : IDL.Null,
    'InternalError' : IDL.Text,
  });
  const upload_chunk_continue_request = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'chunk_id' : IDL.Nat64,
    'item_id' : item_id,
  });
  const item_operation_response_detailed = IDL.Variant({
    'Ok' : IDL.Null,
    'AlreadyUploaded' : IDL.Null,
    'FolderNotEmpty' : IDL.Null,
    'ItemNotFound' : IDL.Null,
    'NotAFile' : IDL.Null,
    'NotAFolder' : IDL.Null,
    'NotRequested' : IDL.Null,
    'PermissionError' : IDL.Null,
    'PendingError' : IDL.Null,
  });
  const upload_file_atomic_request_new = IDL.Record({
    'content' : IDL.Vec(IDL.Nat8),
    'name' : IDL.Text,
    'file_type' : IDL.Text,
    'parent_id' : IDL.Opt(item_id),
    'num_chunks' : IDL.Nat64,
  });
  const upload_file_to_item_request = IDL.Record({
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
    'file_content' : IDL.Vec(IDL.Nat8),
    'item_id' : item_id,
  });
  const VetkdEncryptedKeyResponse = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Nat8),
    'Err' : IDL.Text,
  });
  const VetkdPublicKeyResponse = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Nat8),
    'Err' : IDL.Text,
  });
  const who_am_i_response = IDL.Variant({
    'known_user' : user,
    'unknown_user' : IDL.Null,
  });
  return IDL.Service({
    'create_folder' : IDL.Func(
        [IDL.Text, IDL.Opt(item_id)],
        [IDL.Variant({ 'Ok' : public_item_metadata, 'Err' : IDL.Text })],
        [],
      ),
    'delete_item' : IDL.Func([item_id], [item_operation_response], []),
    'delete_template' : IDL.Func([IDL.Text], [item_operation_response], []),
    'download_file_chunk' : IDL.Func(
        [item_id, IDL.Nat64],
        [download_file_chunk_response],
        ['query'],
      ),
    'get_alias_info' : IDL.Func(
        [IDL.Text],
        [
          IDL.Variant({
            'Ok' : alias_info_response,
            'Err' : get_alias_info_error,
          }),
        ],
        [],
      ),
    'get_group_by_alias' : IDL.Func(
        [IDL.Text],
        [
          IDL.Variant({
            'Ok' : group_info_response,
            'Err' : get_alias_info_error,
          }),
        ],
        [],
      ),
    'get_item_sharers' : IDL.Func(
        [item_id],
        [IDL.Variant({ 'Ok' : IDL.Vec(user), 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_items_shared_with_me' : IDL.Func(
        [],
        [IDL.Vec(public_item_metadata)],
        ['query'],
      ),
    'get_request_groups' : IDL.Func(
        [],
        [IDL.Vec(public_request_group)],
        ['query'],
      ),
    'get_requests' : IDL.Func([], [IDL.Vec(public_item_metadata)], ['query']),
    'get_template' : IDL.Func([IDL.Text], [template_response], ['query']),
    'get_template_names' : IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
    'get_user_canisters' : IDL.Func([], [GetUserCanistersResponse], ['query']),
    'get_user_templates' : IDL.Func([], [IDL.Vec(template)], ['query']),
    'get_users' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Vec(user), 'Err' : IDL.Text })],
        ['query'],
      ),
    'hello_world' : IDL.Func([], [IDL.Text], ['query']),
    'list_folder_contents' : IDL.Func(
        [IDL.Opt(item_id)],
        [
          IDL.Variant({
            'Ok' : IDL.Vec(public_item_metadata),
            'Err' : IDL.Text,
          }),
        ],
        ['query'],
      ),
    'move_item' : IDL.Func(
        [item_id, IDL.Opt(item_id)],
        [item_operation_response],
        [],
      ),
    'multi_request' : IDL.Func(
        [multi_request_input],
        [IDL.Variant({ 'Ok' : multi_request_response, 'Err' : IDL.Text })],
        [],
      ),
    'register_canister' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [RegisterCanisterResponse],
        [],
      ),
    'rename_canister' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [RenameCanisterResponse],
        [],
      ),
    'rename_item' : IDL.Func(
        [item_id, IDL.Text],
        [item_operation_response],
        [],
      ),
    'request_file' : IDL.Func(
        [IDL.Text, IDL.Opt(item_id)],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'revoke_item_share' : IDL.Func(
        [IDL.Principal, item_id],
        [item_operation_response],
        [],
      ),
    'set_user' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [set_user_response],
        [],
      ),
    'share_item' : IDL.Func(
        [IDL.Principal, item_id],
        [item_operation_response],
        [],
      ),
    'unregister_canister' : IDL.Func(
        [IDL.Principal],
        [DeleteCanisterResponse],
        [],
      ),
    'upload_chunk_continue' : IDL.Func(
        [upload_chunk_continue_request],
        [item_operation_response_detailed],
        [],
      ),
    'upload_file_atomic' : IDL.Func(
        [upload_file_atomic_request_new],
        [IDL.Variant({ 'Ok' : item_id, 'Err' : IDL.Text })],
        [],
      ),
    'upload_file_to_item' : IDL.Func(
        [upload_file_to_item_request],
        [item_operation_response_detailed],
        [],
      ),
    'username_exists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
    'vetkd_encrypted_key' : IDL.Func(
        [IDL.Vec(IDL.Nat8), IDL.Opt(item_id)],
        [VetkdEncryptedKeyResponse],
        [],
      ),
    'vetkd_public_key' : IDL.Func([], [VetkdPublicKeyResponse], []),
    'who_am_i' : IDL.Func([], [who_am_i_response], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
