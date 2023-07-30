[![Release](https://img.shields.io/github/v/tag/scotow/cobrust?label=version)](https://github.com/scotow/cobrust/tags)
[![Build Status](https://img.shields.io/github/actions/workflow/status/scotow/cobrust/docker.yml)](https://github.com/scotow/cobrust/actions)


![Banner](banner.png)

## Features

- Game rules customization (grid size, speed, ...)
- Perks and power-ups
- Multiplayer

## Configuration

### Options

```
Usage: cobrust [OPTIONS]

Options:
  -v, --verbose...         Increase logs verbosity (Error (default), Warn, Info, Debug, Trace)
  -a, --address <ADDRESS>  HTTP listening address [default: 127.0.0.1]
  -p, --port <PORT>        HTTP listening port [default: 8080]
  -h, --help               Print help
  -V, --version            Print version
```

### Running locally

```sh
cargo run -- [OPTIONS]
```

### Docker

If you prefer to run `cobrust` as a Docker container, you can either build the image yourself using the Dockerfile available in this repo, or you can use the [image](https://github.com/scotow/cobrust/pkgs/container/cobrust%2Fcobrust) built by the GitHub action.

```
docker run ghcr.io/scotow/cobrust/cobrust:latest
```

### Binding to all interfaces

By default, `cobrust` will only listen on the loopback interface, aka. `127.0.0.1`. If you don't want to host `cobrust` behind a reverse proxy or if you are using the Docker image, you should specify the `0.0.0.0` address by using the `-a | --address` option.