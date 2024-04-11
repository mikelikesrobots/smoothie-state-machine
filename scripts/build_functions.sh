#!/bin/bash
set -euxo

for func in $(ls handlers)
do
  pushd handlers/$func
  cargo lambda build --arm64 --release --output-format zip
  popd
done
