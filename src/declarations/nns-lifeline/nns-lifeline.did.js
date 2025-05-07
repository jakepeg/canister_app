export const idlFactory = ({ IDL }) => {
  const HardResetRootToVersionPayload = IDL.Record({
    'wasm_module' : IDL.Vec(IDL.Nat8),
    'init_arg' : IDL.Vec(IDL.Nat8),
  });
  const UpgradeRootProposalPayload = IDL.Record({
    'wasm_module' : IDL.Vec(IDL.Nat8),
    'module_arg' : IDL.Vec(IDL.Nat8),
    'stop_upgrade_start' : IDL.Bool,
  });
  const LogVisibility = IDL.Variant({
    'controllers' : IDL.Null,
    'public' : IDL.Null,
  });
  const CanisterSettings = IDL.Record({
    'freezing_threshold' : IDL.Opt(IDL.Nat),
    'wasm_memory_threshold' : IDL.Opt(IDL.Nat),
    'controllers' : IDL.Opt(IDL.Vec(IDL.Principal)),
    'reserved_cycles_limit' : IDL.Opt(IDL.Nat),
    'log_visibility' : IDL.Opt(LogVisibility),
    'wasm_memory_limit' : IDL.Opt(IDL.Nat),
    'memory_allocation' : IDL.Opt(IDL.Nat),
    'compute_allocation' : IDL.Opt(IDL.Nat),
  });
  return IDL.Service({
    'hard_reset_root_to_version' : IDL.Func(
        [HardResetRootToVersionPayload],
        [],
        [],
      ),
    'upgrade_root' : IDL.Func([UpgradeRootProposalPayload], [], []),
    'upgrade_root_settings' : IDL.Func([CanisterSettings], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
