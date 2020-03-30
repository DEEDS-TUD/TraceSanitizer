#!/usr/bin/env bash

WIP=$HOME/projects/trace-sanitizer
WIP="$PWD/../.."
BENCHMARK="$(basename "$PWD")"
#DST="$WIP/trace-postprocessor/ressources"
#cd run
DST=$2
ERRORF="$PWD/llfi/error_output/errorfile-run-"
OUTPUTF="$PWD/llfi/std_output/std_outputfile-run-"
FIF="$PWD/llfi/llfi_stat_output/llfi.stat.fi.injectedfaults"

function write_hash {
  shasum $OUTPUTF$1-$2 | cut -d ' ' -f 1 > outhash
}
function insert_front {
  echo "$1
$(cat $2)" > $2
  
}
function copy_trace {
  NAME=${BENCHMARK}_$1
  insert_front "Timestamp,Creator,Createe" mapping
  insert_front "Name,Address,Size" globals
  insert_front "Logical,Physical" logical_mapping
  insert_front "Timestamp,Fault" faultinj
  RET="0"
  if [ -e $ERRORF$2-$3 ]; then
    RET=$(grep 'return code ' $ERRORF$2-$3 | rev | cut -d ' ' -f 1 | rev)
    if [ -z "$RET" ]; then
      RET=$(grep 'Program hang' $ERRORF$2-$3 | rev | cut -d ' ' -f 1 | rev)
    fi
  fi
  
  echo $RET > retc
  insert_front "ReturnCode" retc

  xsv sort -s Timestamp -N -o mapping mapping
  xsv sort -s Timestamp -N -o faultinj faultinj
  
  write_hash $2 $3
  pwd 
  rsync -a mapping $DST/${NAME}_mapping
  rsync -a llfi.stat.* $DST/"${NAME}"
  rsync -a globals $DST/"${NAME}_globals"
  rsync -a logical_mapping $DST/"${NAME}_logical_mapping"
  rsync -a faultinj $DST/"${NAME}_faultinj"
  rsync -a retc $DST/"${NAME}_retc"
  rsync -a $FIF.$2-$3.txt $DST/"${NAME}_FI"
  rsync -a outhash $DST/"${NAME}_outhash" 
}


WORKDIR=$PWD
for flt in $(ls run); do
  if [ $flt != "goldenrun" ]; then
  for e in $(ls run/$flt); do
    cd run/$flt/$e/trace_linear
    tmp=$(echo "$e + $1" | bc)
    f=trace.$flt-$tmp
    copy_trace $f $flt $e
    cd $WORKDIR
  done
fi
done

exit
if [ $2 = "-c" ]; then
  gr=$DST/${BENCHMARK}_trace.goldenrun-0
  fr=$DST/${BENCHMARK}_trace.0-$1

  cp $fr $gr
  cp ${fr}_globals ${gr}_globals
  cp ${fr}_faultinj ${gr}_faultinj
  cp ${fr}_logical_mapping ${gr}_logical_mapping
  cp ${fr}_mapping ${gr}_mapping
  cp ${fr}_retc ${gr}_retc
fi
