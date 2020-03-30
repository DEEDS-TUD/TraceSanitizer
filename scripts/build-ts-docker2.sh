#!/bin/sh

command -v docker >/dev/null ||
{
    echo "Docker binary cannot be found. Please install Docker to use this script."
    exit 1
}

docker build -t trace-sanitizer2 -f docker/Dockerfile2 .
