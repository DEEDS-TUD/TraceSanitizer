#!/usr/bin/env bash

DATA="/home/habib/projects/trace-sanitizer/trace-postprocessor/ressources/fib/fib-pthread"
#docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/svcomp/fib/; LLFI_FI_OFF=BLA ./run-fi.sh fib" && ./move_traces.sh && ./linearize.sh && ./start.sh 0 "$DATA/gr/raw-traces"
docker exec -it sani_cont sh -c "cd /home/llfi/target-programs/svcomp/fib/; ./run-fi.sh fib" && ./move_traces.sh && ./linearize.sh && ./start.sh 0 "$DATA/fi/raw-traces"

