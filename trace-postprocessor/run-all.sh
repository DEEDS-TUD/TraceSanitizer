#!/usr/bin/env bash
echo starting at $(date)
#export VDATA=/home/habib/tsan-data/tmp
gr_reps=5
gr_rep=5
fi_rep=5
if test "$#" -eq 3; then
  gr_reps=$1
  gr_rep=$2
  fi_rep=$3
fi
for bench in quicksort blackscholes kmeans pca swaptions; do
  cargo run --release -- $bench $gr_reps -o -s -l && cargo run --release -- $bench $gr_rep -o && 
  cargo run --release -- $bench $reps -o -f 0
  for f in $(seq 1 5); do
    cargo run --release -- $bench $fi_rep -o -a -f $f || break
  done
done
echo finished at $(date)
