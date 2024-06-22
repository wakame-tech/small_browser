curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -y

cd ./renderer && wasm-pack build --dev . --target web --out-dir ./web/src/wasm
cd ./web/ && npm run build
