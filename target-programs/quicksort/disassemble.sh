#!/usr/bin/env bash
llvm-dis llfi/$1-profiling.bc && mv llfi/$1-profiling.ll run
llvm-dis llfi/$1-llfi_index.bc && mv llfi/$1-llfi_index.ll run
