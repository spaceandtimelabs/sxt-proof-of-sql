# Set up your environment

1. Only `linux` systems are supported.
2. Only `x86_64` architectures are supported.
3. Use a docker container with `ubuntu20.04` or some virtual machine. The files in the `ci/docker` directory specify the exact setup that you will need.

# Using the Docker containers

If you have a Linux machine, a Docker installed, and a GPU available, run the following command:

```
bash ci/docker/run_docker_with_gpu.sh
```

In case you don't have a GPU available, run:

```
bash ci/docker/run_docker_with_cpu.sh
```

Bear in mind that you must execute the above commands from the root proofs directory.

But after executing them, you can test if everything is working properly:

```
cargo test
```
