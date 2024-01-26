# Aptos Summits: Move

## Mainnet
To mint to someone, do this:
```
aptos move run --profile mainnet --function-id 0x`yq .profiles.mainnet.account < .aptos/config.yaml`::summits_token::mint_to --args address:0x123
```

## Testnet
To publish to testnet at a new address:
```
yes '' | aptos init --profile testnetpublish --assume-yes --network testnet && aptos move publish --profile testnetpublish --assume-yes --named-addresses addr=testnetpublish
```

Create the collection:
```
aptos move run --profile testnetpublish --assume-yes --function-id 0x`yq .profiles.testnetpublish.account < .aptos/config.yaml`::summits_collection::create
```

## Generating schema
Build the Aptos CLI from the correct aptos-core branch.
```
cd ~/a/core
git checkout banool/rust-move-codegen
cargo build -p aptos
```

Generate the GraphQL schema representation of the module ABI.
```
~/a/core/target/debug/aptos move generate schema --named-addresses addr=0x123 --schema-path ./
```

To regenerate the types for the backend run this here.
```
~/a/core/target/debug/aptos move generate rust --named-addresses addr=0x123 --generate-to ../api/move-types/src/
mv ../api/move-types/src/mod.rs ../backend/move-types/src/lib.rs
```

To regenerate the types for the frontend run this from within `frontend/`.
```
pnpm generate-move
```
