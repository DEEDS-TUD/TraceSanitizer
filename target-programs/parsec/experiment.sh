#!/usr/bin/env bash

gr_lims=0
gr_lim=0
fi_lim=0
bench=swaptions

if test "$#" -eq 4; then
  bench=$1 
  gr_lim=$(( ( $3 - 1 )  / 6 ))
  gr_lims=$(( ( $2 - 1 ) / 6 ))
  fi_lim=$(( $4 - 1 ))
fi

DATADST="$PWD/../../trace-postprocessor/ressources"
DATAS="$DATADST/$bench/$bench-serial"
DATAP="$DATADST/$bench/$bench-pthread"

gr_rep=$(seq 0 1 $gr_lim)
gr_reps=$(seq 0 1 $gr_lims)
fi_rep=$(seq 0 1 $fi_lim)

date
WIP=$(pwd)

for r in $gr_reps; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/parsec/; LLFI_FI_OFF=BLA ./run.sh $bench serial" && cd pkgs/apps/$bench/ && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAS/gr/raw-traces"
done

for r in $gr_rep; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/parsec/; LLFI_FI_OFF=BLA ./run.sh $bench pthreads" && cd pkgs/apps/$bench/ && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAP/gr/raw-traces"
done

for r in $fi_rep; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/parsec/; ./run.sh $bench pthreads" && cd pkgs/apps/$bench/ && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAP/fi/raw-traces"
done

