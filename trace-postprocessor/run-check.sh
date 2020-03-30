#!/usr/bin/env bash
echo starting at $(date)
#export VDATA="re"
reps=1
for bench in quicksort blackscholes pca kmeans swaptions; do
  for iter in $(seq 1 10); do
    RUST_BACKTRACE=1 cargo run --release -- $bench $reps -o -c
  done
done
echo finished at $(date)
