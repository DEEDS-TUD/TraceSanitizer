#!/usr/bin/env bash

command -v docker >/dev/null ||
{
    echo "Docker binary cannot be found. Please install Docker to use this script."
    exit 1
}


# location of this script and its absolute folder
THIS_SCRIPT="${BASH_SOURCE[0]}"
THIS_DIR="$(cd "$(dirname "$THIS_SCRIPT")" &> /dev/null && pwd)"

docker run \
  --name sani_cont \
  -v $THIS_DIR/../llfi:/home/llfi/llfisrc:rw \
  -v $THIS_DIR/../target-programs:/home/llfi/target-programs:rw \
  -it --rm \
  trace-sanitizer
