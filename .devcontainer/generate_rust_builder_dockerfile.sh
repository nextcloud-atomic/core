#!/usr/bin/bash

set -eux

tmpdir="$(mktemp -d)"
echo "working in directory $tmpdir"
trap 'rm -rf $tmpdir' EXIT

(
cd "$tmpdir"

git clone https://github.com/rust-lang/docker-rust.git .
sed -i 's/debian_variants = \[/debian_variants = \[ DebianVariant("trixie", debian_lts_arches + debian_non_lts_arches),/' ./x.py
python ./x.py update
cat ./stable/trixie/slim/Dockerfile
)

