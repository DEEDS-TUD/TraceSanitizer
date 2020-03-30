# Phoenix Benchmarks

These are some of the phoenix benchmarks as used in the dthreads paper. More precisely, they are some of the phoenix test programs.

Dthreads Repo: https://github.com/plasma-umass/dthreads (eval/tests)

Phoenix Repo: https://github.com/kozyraki/phoenix (phoenix-2.0/tests)

## Input Data

Some of the test programs require input data. The original phoenix input data is linked in the readme file of the phoenix repo. We are using a reduced dataset.

## Test Programs

- kmeans
  Working. No input data needed.
- matrix_multiply
  Working. No input data needed.
- pca
  Working. No input data needed.
- reverse_index
  Working. Needs input data. We have the bottom 50 and bottom 100 websites as datasets (note that git does not retain empty dirs, which slightly reduces the datasets).
- string_match
  Working. Needs input data. We have a 10KB and 100KB data file.
- word_count
  Working. Needs input data. We have a 10KB and 100KB data file.

## LLFI

The following steps are required to build and execute the pthreads variants of the test programs with LLFI instrumentation/fault injection.

1. Change into the subdirectory of the test program
2. Run `make llfi-seq` for the sequential version and  `make llfi-pthread` for the parallel version. Note that you cannot have both at the same time.
   This will produce the original executable as well as the profiling and FI executables in the llfi subfolder. Bitcode files are retained in the llfi subfolder.
3. Run `env EXEC_MODE=seq ./run-llfi.sh` to execute the profiling run followed by the FI campaign when using the sequential version. Run  `env EXEC_MODE=pthread ./run-llfi.sh`  for the parallel version. Note the both the program input and the FI campaign are reduced in size for faster feedback and debugging.
4. Enjoy the huge pile of data that has been produced and stored away in subfolders of the llfi folder.

