A web app that has a button to control a GPIO pin.

### Building within Docker
```sh
docker run --name gpio-toggle-build -it --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/src -w /src --dns=1.1.1.1 rust:slim-stretch cargo build --release
```
