set -euxo pipefail

sudo apt-get update
sudo apt-get install -y python3-dev python3-pip python3-venv libclang-dev
sudo python3 -m pip install cffi virtualenv pipx pip -U

pipx ensurepath
pipx install uniffi-bindgen
pipx install cargo-deny

pipx install poetry
poetry shell --no-interaction && poetry install --no-interaction

rustup target add wasm32-wasi
curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin