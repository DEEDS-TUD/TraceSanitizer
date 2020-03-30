#!/usr/bin/env bash
#opt -load /home/llfi/llfi/llvm_passes/llfi-passes.so -o example-instr.bc -testpass < example.ll
./cleanup.sh
mkdir run && ~/llfi/bin/instrument $1.ll && cp llfi/$1-profiling.exe . && ./$1-profiling.exe && mv llfi.stat.trace* run

./disassemble.sh $1
