# Docker Images

The docker images for x86_64 are built with AVX2 SIMD extensions, while the images for aarch64 are built with NEON SIMD extensions. These must be available on your machine. If your machine does not support these extensions, you must build the docker image with the build-arg `simd_disabled=true`.

To build the images, simply run:

```
docker build . -f docker/Dockerfile
```

from the project root.

To build the images without SIMD vector extensions, run 

```
docker build . -f docker/Dockerfile --build-arg simd_disabled=true
```

from the project root and run any subsequent docker commands on the resultant image.

To ensure you are using the latest version of piestream image,

```
# Ensure piestream image is of latest version
docker pull ghcr.io/piestreamlabs/piestream:latest
```

To start a piestream playground, run

```
# Start playground
docker run -it --pull=always -p 5505:5505 -p 5691:5691 ghcr.io/piestreamlabs/piestream:latest playground
```

To start a piestream cluster, run

```
# Start all components
docker-compose up
```

It will start a minio, a meta node, a compute node, a frontend, a compactor, a prometheus and a redpanda instance.

To clean all data, run:

```
docker-compose down -v
```

For piestream kernel hackers, we always recommend using [risedev](../src/risedevtool/README.md) to start the full cluster, instead of using docker images.
See [CONTRIBUTING](../CONTRIBUTING.md) for more information.

# Generate docker-compose.yml

```bash
./risedev compose
```
