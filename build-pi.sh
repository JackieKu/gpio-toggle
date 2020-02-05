#!/bin/sh
exec docker run -it --rm -u root --name rust-raspberry-build -v $PWD:/home/cross/project ragnaroek/rust-raspberry:1.40.0 build "$@"
