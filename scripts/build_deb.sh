#!/bin/bash

VERSION="$1"
RELEASE="$2"

printf "Simple authentication service\n" > description-pak
checkinstall --pkgversion ${VERSION} --pkgrelease ${RELEASE} -y
