#!/bin/bash

source env.sh

parsecmgmt -c llfi-$2 -p $1 -a uninstall
parsecmgmt -c llfi-$2 -p $1 -a clean
parsecmgmt -c llfi-$2 -p $1 -a build
#parsecmgmt -c llfi-$2 -p $1 -a run -n 4
parsecmgmt -c llfi-$2 -p $1 -i llfi-test -a run -n 3

#cd pkgs/apps/$1/run
