#!/bin/sh
# Rebuild our model from the pinned, commercially usable CC BY-SA DDT data.
# Requires Cargo, a C++ compiler, curl and sha256sum (on macOS: install coreutils).
set -eu
cd "$(dirname "$0")/.."
train_dir="${DANISH_TRAINING_DIR:-$PWD/data/training}"
mkdir -p "$train_dir/tmp"
export TMPDIR="$train_dir/tmp"
cargo build --locked -j 2
# Locate the source package and static library that Cargo actually built.
# A JSON manifest containing all source URLs/checksums is kept under models/.
registry_root="${CARGO_HOME:-$HOME/.cargo}/registry/src"
udpipe_src=$(find "$registry_root" -maxdepth 3 -type d -name udpipe-rs-0.2.0 -print -quit)
udpipe_lib=$(find "${CARGO_TARGET_DIR:-target}/debug/build" -path '*/udpipe-rs-*/out/libudpipe.a' -print -quit)
test -n "$udpipe_src" && test -n "$udpipe_lib"
"${CXX:-c++}" -O2 -std=c++11 -I "$udpipe_src/vendor/udpipe/src" \
 "$udpipe_src/vendor/udpipe/src/udpipe.cpp" \
 "$udpipe_src/vendor/udpipe/src/version/version.cpp" \
 "$udpipe_lib" -o "$train_dir/udpipe"
base=https://raw.githubusercontent.com/UniversalDependencies/UD_Danish-DDT/5880293fa36da950fa833fe0b52cb7d21882c481
curl --fail --location "$base/da_ddt-ud-train.conllu" -o "$train_dir/da_ddt-ud-train.conllu"
curl --fail --location "$base/da_ddt-ud-dev.conllu" -o "$train_dir/da_ddt-ud-dev.conllu"
(
 cd "$train_dir"
 echo '65730e35b07d1e944ee9ae1d344b5c84746cf0aeadf5faf0550059b4ded86fb6  da_ddt-ud-train.conllu' | sha256sum -c -
 echo 'd714b22776ad60fbdb6e4ffc15a5c3ffc162705617567d3cf2b09cc6b51e800a  da_ddt-ud-dev.conllu' | sha256sum -c -
)
"$train_dir/udpipe" --train --tokenizer=model=generic --tagger=iterations=20 \
 --parser=iterations=10 --heldout="$train_dir/da_ddt-ud-dev.conllu" \
 "$train_dir/danish-ddt.udpipe" "$train_dir/da_ddt-ud-train.conllu" \
 > "$train_dir/train.log" 2>&1
printf 'Model: %s/danish-ddt.udpipe\nLog: %s/train.log\n' "$train_dir" "$train_dir"
# Do not replace the shipped model automatically: compare regressions first.
