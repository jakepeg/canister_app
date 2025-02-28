dfx stop
rm -rf .dfx
dfx start --clean --background
dfx generate
dfx canister create vetkd_system_api --specified-id nn664-2iaaa-aaaao-a3tqq-cai
or
dfx canister create --all
dfx deploy vetkd_system_api
dfx deploy
