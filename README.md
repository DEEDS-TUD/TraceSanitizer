# Trace Sanitizer

Execution traces are useful to analyze program behavior at run time.
Especially differential analyses of execution traces suffer from execution non-determinism, as it hampers a direct comparison across traces.
To mitigate this issue we develop Trace Sanitizer as a framework to eliminate execution trace differences.

## Evaluation Environment

To evaluate Trace Sanitizer we apply it to execution traces collected from repeated executions of different programs instrumented via LLFI.
In order to facilitate reproducibility of our results we specify our evaluation environment as a Docker file, which can be used to create a docker container for reproducing our results.
To build the container, please first install docker and then execute the script `build-ts-docker.sh` in the scripts directory of this repository.
Please make sure to start the script *inside* the scripts directory and that you have your favorite beverage in reach. This may take a while...

If the docker build process fails to fetch keys from pgp.mit.edu, please repeatedly invoke the script.
This is an intermittent error that happens from time to time and we are too busy right now to write a more robust alternative script that automatically attempts re-connections.

After building the docker image you can start it via

```
docker run -v <path-to-llfi-sources>:/home/llfi/llfisrc:rw -v <path-to-target-programs>:/home/llfi/target-programs:rw -it --rm trace-sanitizer
```
Alternatively you can run the script `run-container.sh` located under the scripts folder. Make sure you run the script from the scripts folder. The script will run the container in interactive, non-persistent mode and mount the taret programs as well as our `llfi` fork in the home directory.

The default user is `llfi`, its password is `root` and it is a member of the `sudo` group.

In `llfi`'s home directory we have the source directory of llfi and llvm along with a build directory for `llvm` and our benchmarks under `target-programs`.

After you start the container, make sure you run the setup script in the the llfisrc source. If you haven't modified anything in the paths, the default paths used in the script should work just fine and a build `llfi` folder will be generated in the `llfi`'s home directory.

You can build a benchmark with

```
clang -S -emit-llvm <program-name>.c
```

Instrument with

```
~/llfi/bin/instrument <program-name>.ll
```

and profile with

```
cp llfi/<program-name>-profiling.exe .
./<program-name>-profiling.exe
```

The execution traces can be found in llfi.stat.trace.txt. Since the complete `target-programs` folder is mounted into the container, the generated traces are accessible (even after terminating the container) in the `target-programs` folder in the host machine.
