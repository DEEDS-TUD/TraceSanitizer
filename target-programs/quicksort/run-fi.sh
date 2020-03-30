#!/usr/bin/env bash
#opt -load /home/llfi/llfi/llvm_passes/llfi-passes.so -o example-instr.bc -testpass < example.ll
./cleanup.sh
clang -S -emit-llvm $1.c && mkdir -p run/goldenrun/0 && ~/llfi/bin/instrument $1.ll && ~/llfi/bin/profile llfi/$1-profiling.exe && ~/llfi/bin/injectfault llfi/$1-faultinjection.exe
