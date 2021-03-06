#!/usr/bin/env bash
WIP="$PWD/../../.."
#echo "Timestamp,Creator,Createe" > run/mapping.txt
#echo "Name,Address,Size" > run/globals.txt
#grep Mapping run/llfi.stat.trace* | cut -d ' ' -f 2 >> run/mapping.txt
#cd run
CURR_DIR=$PWD
for flt in $(ls $CURR_DIR/run/); do
  echo Fault: $flt
  for e in $(ls $CURR_DIR/run/$flt); do
    echo Experiment: $e
    cd $WIP
    echo $CURR_DIR/run/$flt/$e
    ./trace-formatter.py $CURR_DIR/run/$flt/$e
    cd $CURR_DIR
    xsv cat rows run/$flt/$e/trace*/llfi.stat.trace* | xsv sort -s Timestamp -N -o run/$flt/$e/trace_linear/llfi.stat.trace_linear
    rm -r run/$flt/$e/trace*-*
  done

done
#cp run/trace*/llfi.stat.* run/trace_linear
#xsv cat rows run/trace*/llfi.stat.trace* | xsv sort -s Timestamp -N -o run/trace_linear/llfi.stat.trace_linear

