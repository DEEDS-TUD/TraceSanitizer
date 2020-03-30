#!/usr/bin/env bash
#DATADST="$HOME/projects/trace-sanitizer/trace-postprocessor/ressources"
DATADST="$PWD/../../trace-postprocessor/ressources"
DATAS="$DATADST/quicksort/quicksort-serial"
DATAP="$DATADST/quicksort/quicksort-pthread"

gr_lims=0
gr_lim=0
fi_lim=0

if test "$#" -eq 3; then
  gr_lim=$(( ( $2 - 1 )  / 6 ))
  gr_lims=$(( ( $1 - 1 ) / 6 ))
  fi_lim=$(( $3 - 1 ))
fi


gr_rep=$(seq 0 1 $gr_lim)
gr_reps=$(seq 0 1 $gr_lims)
fi_rep=$(seq 0 1 $fi_lim)
date
WIP=$(pwd)

for r in $gr_reps; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/quicksort/; LLFI_FI_OFF=BLA ./run-fi.sh quicksort" && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAS/gr/raw-traces"
done
exit
for r in $gr_rep; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/quicksort/; LLFI_FI_OFF=BLA ./run-fi.sh quicksort-pthread" && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAP/gr/raw-traces"
done

for r in $fi_rep; do
  cd $WIP
  docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/quicksort/; ./run-fi.sh quicksort-pthread" && ./move_traces.sh && ./linearize.sh && ./start.sh $r "$DATAP/fi/raw-traces"
done
