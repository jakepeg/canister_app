export const idlFactory = ({ IDL }) => {
  const file_id = IDL.Nat64;
  const share_file_response = IDL.Variant({
    'ok' : IDL.Null,
    'permission_error' : IDL.Null,
  });
  const found_file = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
  });
  const download_file_response = IDL.Variant({
    'found_file' : found_file,
    'permission_error' : IDL.Null,
    'not_uploaded_file' : IDL.Null,
    'not_found_file' : IDL.Null,
  });
  const user = IDL.Record({
    'username' : IDL.Text,
    'public_key' : IDL.Vec(IDL.Nat8),
    'ic_principal' : IDL.Principal,
  });
  const get_alias_info_response = IDL.Variant({
    'Ok' : IDL.Record({
      'user' : user,
      'file_name' : IDL.Text,
      'file_id' : file_id,
    }),
    'Err' : IDL.Variant({ 'not_found' : IDL.Null }),
  });
  const file_info = IDL.Record({
    'alias' : IDL.Text,
    'file_name' : IDL.Text,
    'file_id' : file_id,
  });
  const group_info = IDL.Record({
    'files' : IDL.Vec(file_info),
    'requester' : user,
    'group_id' : IDL.Nat64,
    'group_name' : IDL.Text,
  });
  const file_status = IDL.Variant({
    'partially_uploaded' : IDL.Null,
    'pending' : IDL.Record({ 'alias' : IDL.Text, 'requested_at' : IDL.Nat64 }),
    'uploaded' : IDL.Record({ 'uploaded_at' : IDL.Nat64 }),
  });
  const file_metadata = IDL.Record({
    'file_status' : file_status,
    'group_alias' : IDL.Opt(IDL.Text),
    'file_name' : IDL.Text,
    'shared_with' : IDL.Vec(user),
    'group_name' : IDL.Text,
    'file_id' : file_id,
  });
  const public_request_group = IDL.Record({
    'files' : IDL.Vec(file_metadata),
    'name' : IDL.Text,
    'created_at' : IDL.Nat64,
    'group_id' : IDL.Nat64,
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
  const get_users_response = IDL.Variant({
    'permission_error' : IDL.Null,
    'users' : IDL.Vec(user),
  });
  const multi_request_input = IDL.Record({
    'file_names' : IDL.Vec(IDL.Text),
    'save_as_template' : IDL.Bool,
    'group_name' : IDL.Text,
  });
  const multi_request_response = IDL.Record({
    'group_alias' : IDL.Text,
    'group_id' : IDL.Nat64,
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
  const upload_file_request = IDL.Record({
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
    'file_content' : IDL.Vec(IDL.Nat8),
    'file_id' : file_id,
  });
  const upload_file_error = IDL.Variant({
    'not_requested' : IDL.Null,
    'already_uploaded' : IDL.Null,
  });
  const upload_file_response = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : upload_file_error,
  });
  const upload_file_atomic_request = IDL.Record({
    'content' : IDL.Vec(IDL.Nat8),
    'name' : IDL.Text,
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
  });
  const upload_file_continue_request = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'chunk_id' : IDL.Nat64,
    'file_id' : file_id,
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
    'known_user' : IDL.Record({ 'username' : IDL.Text }),
    'unknown_user' : IDL.Null,
  });
  return IDL.Service({
    'delete_file' : IDL.Func([file_id], [share_file_response], []),
    'delete_template' : IDL.Func([IDL.Text], [], []),
    'download_file' : IDL.Func(
        [file_id, IDL.Nat64],
        [download_file_response],
        ['query'],
      ),
    'get_alias_info' : IDL.Func(
        [IDL.Text],
        [get_alias_info_response],
        ['query'],
      ),
    'get_file_owner_principal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Vec(IDL.Nat8), 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_group_by_alias' : IDL.Func(
        [IDL.Text],
        [
          IDL.Variant({
            'Ok' : group_info,
            'Err' : IDL.Variant({ 'not_found' : IDL.Null }),
          }),
        ],
        ['query'],
      ),
    'get_request_groups' : IDL.Func(
        [],
        [IDL.Vec(public_request_group)],
        ['query'],
      ),
    'get_requests' : IDL.Func([], [IDL.Vec(file_metadata)], ['query']),
    'get_shared_files' : IDL.Func([], [IDL.Vec(file_metadata)], ['query']),
    'get_template' : IDL.Func([IDL.Text], [template_response], ['query']),
    'get_template_names' : IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
    'get_user_canisters' : IDL.Func([], [GetUserCanistersResponse], ['query']),
    'get_user_templates' : IDL.Func([], [IDL.Vec(template)], ['query']),
    'get_users' : IDL.Func([], [get_users_response], ['query']),
    'hello_world' : IDL.Func([], [IDL.Text], []),
    'multi_request' : IDL.Func(
        [multi_request_input],
        [multi_request_response],
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
    'rename_file' : IDL.Func([file_id, IDL.Text], [share_file_response], []),
    'request_file' : IDL.Func([IDL.Text], [IDL.Text], []),
    'revoke_share' : IDL.Func(
        [IDL.Principal, file_id],
        [share_file_response],
        [],
      ),
    'set_user' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [set_user_response],
        [],
      ),
    'share_file' : IDL.Func(
        [IDL.Principal, file_id],
        [share_file_response],
        [],
      ),
    'share_file_with_users' : IDL.Func(
        [IDL.Vec(IDL.Principal), file_id],
        [],
        [],
      ),
    'unregister_canister' : IDL.Func(
        [IDL.Principal],
        [DeleteCanisterResponse],
        [],
      ),
    'upload_file' : IDL.Func([upload_file_request], [upload_file_response], []),
    'upload_file_atomic' : IDL.Func(
        [upload_file_atomic_request],
        [file_id],
        [],
      ),
    'upload_file_continue' : IDL.Func([upload_file_continue_request], [], []),
    'username_exists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
    'vetkd_encrypted_key' : IDL.Func(
        [IDL.Vec(IDL.Nat8), IDL.Opt(IDL.Nat64)],
        [VetkdEncryptedKeyResponse],
        [],
      ),
    'vetkd_public_key' : IDL.Func([], [VetkdPublicKeyResponse], []),
    'who_am_i' : IDL.Func([], [who_am_i_response], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
