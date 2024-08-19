#!/bin/bash

set -e

SCRIPT_DIR="$(dirname $0)"

ssh -o BatchMode=yes "$DEPLOY_DEB_HOST" "rm -rf $DEPLOY_DEB_PACKAGE_DIR/incoming && mkdir -p $DEPLOY_DEB_PACKAGE_DIR/incoming"

for PACKAGE in $DEPLOY_DEB_PACKAGES; do
    scp -o BatchMode=yes "$PACKAGE" "$DEPLOY_DEB_HOST:$DEPLOY_DEB_PACKAGE_DIR/incoming/"
done

ssh -o BatchMode=yes \
    "$DEPLOY_DEB_HOST" \
    "finish() { \
        rm -rf \"$DEPLOY_DEB_PACKAGE_DIR/incoming\"; \
     }; \
     \
     trap finish EXIT; \
     \
     cd \"$DEPLOY_DEB_PACKAGE_DIR/repos/$DEPLOY_DEB_REPO\" \
        && reprepro -V includedeb $DEPLOY_DEB_CODENAME \"$DEPLOY_DEB_PACKAGE_DIR/incoming/\"*.deb"
