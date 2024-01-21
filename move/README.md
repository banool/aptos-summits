# Aptos Summits: Move

## Helpful test commands

First:
```
export NETWORK=local
```

Publish dport std (run from within aptos-dport-std):
```
curl -X POST "localhost:8081/mint?amount=1000000000&address=`yq .profiles.$NETWORK.account < .aptos/config.yaml`" && sleep 1 && aptos move publish --profile $NETWORK --assume-yes
```

Create the local account on chain:
```
aptos init --network local --private-key 0x9bd759c66531662ad734d501db59809d9a803b0827696f7330dbbe42a183e68e --profile $NETWORK --assume-yes
```

Publish the package. This also sets up the collection and fungible token:
```
aptos move publish --named-addresses addr=$NETWORK --assume-yes --profile $NETWORK
```

Mint PNT to an account:
```
aptos move run --assume-yes --function-id $NETWORK::paint_fungible_asset::mint --args address:`yq .profiles.$NETWORK.account < .aptos/config.yaml` u64:200000 --profile $NETWORK
```

Create a collection (where it costs 12 PNT to draw a pixel):
```
aptos move run --assume-yes --function-id $NETWORK::canvas_token::create --args string:"Numero Uno" string:"Where it all begins" u64:250 u64:200 u64:0 u64:0 u64:100 u64:2 u64:300 u8:255 u8:255 u8:255 bool:true bool:true --profile $NETWORK
```

Get the address of the object created by the previous command:
```
curl localhost:8080/v1/transactions/by_hash/0xe6a7b044015180e07e5878dc8d87729010fa25241d76ea34b2ebc003e9b64e6b | jq -r .events[0].data.token
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
