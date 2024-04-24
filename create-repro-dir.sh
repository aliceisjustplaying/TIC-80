#!/bin/bash
mkdir repro && cd repro || exit 1
for i in {001..100}; do
  touch "test-$i.tic"
done
