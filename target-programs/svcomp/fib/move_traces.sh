#!/usr/bin/env bash

for f in $(ls llfi/baseline/llfi.stat.trace*.txt); do
    tmp="${f%*.prof.txt}"
    #cp $f run/goldenrun/0/$(basename -- "$tmp")
    mv $f run/goldenrun/0/$(basename -- "$tmp")

done
for d in $(find llfi/llfi_stat_output -name llfi.stat.trace* | cut -d '.' -f 4 | sort -u); do
  #echo $d
  faultm=$(echo $d | cut -d '-' -f 1)
  mkdir -p run/$faultm
  e=$(echo $d | cut -d '-' -f 2)
  #echo $e
  mkdir -p run/$faultm/$e
  for f in llfi/llfi_stat_output/llfi.stat.trace*.$d.txt; do
    #echo $f
    tmp="${f%*.$d.txt}"
    #cp $f run/$faultm/$e/$(basename -- "$tmp")
    mv $f run/$faultm/$e/$(basename -- "$tmp")

  done
done


