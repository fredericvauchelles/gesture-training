#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=$(realpath "$SCRIPT_DIR/../..")

docker run --rm -v "$ROOT_DIR":/data -it "$(docker build -q $SCRIPT_DIR/build-docker)"

flatpak-builder --force-clean --user --install-deps-from=flathub --repo=$ROOT_DIR/target/repo --install $ROOT_DIR/target/flatpak $ROOT_DIR/org.fredericvauchelles.GestureTraining.yml
