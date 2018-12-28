#!/bin/bash

VERSION="$1"
RELEASE="$2"

. ~/.cargo/env

cargo build --release

printf "Simple authentication service\n" > description-pak
checkinstall --pkgversion ${VERSION} --pkgrelease ${RELEASE} -y
