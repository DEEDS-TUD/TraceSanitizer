#!/bin/bash

source env.sh

parsecmgmt -c llfi-pthreads -p blackscholes -a uninstall
parsecmgmt -c llfi-pthreads -p blackscholes -a clean
parsecmgmt -c llfi-pthreads -p blackscholes -a build
parsecmgmt -c llfi-pthreads -p blackscholes -a run -n 2

cd pkgs/apps/blackscholes/run
