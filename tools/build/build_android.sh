#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=$(realpath "$SCRIPT_DIR/../..")

export ANDROID_HOME=${ANDROID_SDK_HOME:-$(realpath ~/Android/Sdk)}
export ANDROID_NDK_HOME=${ANDROID_SDK_HOME:-$(realpath ~/Android/Sdk/ndk)}

cd "$ROOT_DIR"/gesture-training || exit

x build --platform android --arch arm64

# x run --device adb:bedd7012 --arch arm64
# x build --device adb:bedd7012 --arch arm64