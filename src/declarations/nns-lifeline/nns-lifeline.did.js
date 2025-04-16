export const idlFactory = ({ IDL }) => {
  const CanisterStatusType = IDL.Variant({
    'stopped' : IDL.Null,
    'stopping' : IDL.Null,
    'running' : IDL.Null,
  });
  const DefiniteCanisterSettings = IDL.Record({
    'controllers' : IDL.Vec(IDL.Principal),
  });
  const CanisterStatusResult = IDL.Record({
    'status' : CanisterStatusType,
    'memory_size' : IDL.Nat,
    'cycles' : IDL.Nat,
    'settings' : DefiniteCanisterSettings,
    'module_hash' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  return IDL.Service({
    'canister_status' : IDL.Func(
        [IDL.Record({ 'canister_id' : IDL.Principal })],
        [CanisterStatusResult],
        [],
      ),
    'upgrade_root' : IDL.Func(
        [
          IDL.Record({
            'wasm_module' : IDL.Vec(IDL.Nat8),
            'module_arg' : IDL.Vec(IDL.Nat8),
            'stop_upgrade_start' : IDL.Bool,
          }),
        ],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
