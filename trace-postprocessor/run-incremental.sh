#!/usr/bin/env bash
echo starting at $(date)
export VDATA=/home/habib/tsan-data
reps=50
max=5100
for bench in kmeans; do
  for off in $(seq 0 $reps $max); do
    for f in $(seq 0 5); do
      cargo run --release -- $bench $reps -o -a -f $f -i $off || break
    done
  done 

  #for off in $(seq 0 $reps $max); do
    #for f in $(seq 0 5); do
    #  cargo run --release -- $bench $reps -o -a -s -f $f -i $off || break
    #done
  #done
done
echo finished at $(date)
