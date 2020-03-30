#!/usr/bin/env bash
#opt -load /home/llfi/llfi/llvm_passes/llfi-passes.so -o example-instr.bc -testpass < example.ll
#./cleanup.sh
make clean && make llfi-$1 && EXEC_MODE=$1 ./run-llfi.sh
#~/llfi/bin/instrument $1.ll && ~/llfi/bin/profile llfi/$1-profiling.exe && ~/llfi/bin/injectfault llfi/$1-faultinjection.exe
