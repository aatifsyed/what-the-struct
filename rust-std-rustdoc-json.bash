#!/usr/bin/env bash
set -euxo pipefail

time docker build --tag rust-std-rustdoc-json - < rust-std-rustdoc-json.Dockerfile

vendor_dir=vendor
mkdir --parents "$vendor_dir"

docker run --rm -it rust-std-rustdoc-json cat build/x86_64-unknown-linux-gnu/json-doc/json-doc.tar \
    | tar --extract --verbose --file - --directory "$vendor_dir"
