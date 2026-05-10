<<<<<<< HEAD
# elecium-substrate
Ce projet a pour but de rendre auditable de système de vote electronique securisé "elecium". Il rend disponible le code source du pallet-vote qui decrit la logique des transactions de votes et permet de verifier que les hash du runtime On-Chain (disponible en blockchain) et celui de ce projet d'audite. Ainsi, le code de la blockchain est fiable.
=======

Ce guide permet à n'importe qui de vérifier que le code source
correspond exactement au runtime qui tourne sur la blockchain.

## Prérequis

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
rustup target add wasm32-unknown-unknown

# Outils
sudo apt install -y git python3 curl
```

## Étape 1 — Cloner le dépôt

```bash
git clone https://github.com/TON_USERNAME/elecium-node
cd elecium-node
git checkout v1.0.0  # tag de la version déployée
```

## Étape 2 — Compiler le runtime

```bash
cargo build --release -p solochain-template-runtime
```

Le WASM compilé se trouve ici :
target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm

## Étape 3 — Télécharger le runtime depuis la blockchain

```bash
curl -s -X POST http://RPC_IP:9944 \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"state_getStorage","params":["0x3a636f6465"]}' \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
hex_val = data['result'][2:]  # enlever 0x
with open('runtime_onchain.wasm', 'wb') as f:
    f.write(bytes.fromhex(hex_val))
print('Runtime téléchargé:', len(bytes.fromhex(hex_val)), 'bytes')
"
```

## Étape 4 — Comparer les hash

```bash
echo '=== Hash du runtime ON-CHAIN ==='
sha256sum runtime_onchain.wasm

echo '=== Hash du runtime COMPILÉ ==='
sha256sum target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm
```

Si les deux hash sont **identiques**, le code source correspond
exactement au runtime qui tourne sur la blockchain.

## Étape 5 — Vérifier le pallet Vote

Le pallet Vote est dans `pallets/pallet-vote/src/lib.rs`.

Points clés à auditer :
- La vérification ZK est bien appelée via host function
- Le nullifier est bien vérifié avant d'accepter un vote
- Le double vote est bien empêché via `NullifierUsed`
- Seul `AdminOrigin` (sudo) peut créer/fermer une élection

## Nœuds RPC publics

- `ws://RPC1_IP:9944`
- `ws://RPC2_IP:9944`

## Genesis hash
0x0f02dce49c6ae9cd867b32f953b844e938db01a9b3425f5a0bf5d3501ce44de1

Vérifiable avec :
```bash
curl -s -X POST http://RPC_IP:9944 \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[0]}' \
  | python3 -m json.tool
```
EOF
