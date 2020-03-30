#!/bin/bash

source env.sh

parsecmgmt -c llfi-serial -p $1 -a uninstall
parsecmgmt -c llfi-serial -p $1 -a clean
parsecmgmt -c llfi-serial -p $1 -a build
parsecmgmt -c llfi-serial -p $1 -a run -n 1
#parsecmgmt -c llfi-serial -p $1 -i simdev -a run -n 1 

cd pkgs/apps/$1/run
