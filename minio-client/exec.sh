#!/bin/bash

set -euo pipefail

mc alias set $MINIO_ALIAS $MINIO_HOST $MINIO_ACCESS_KEY $MINIO_SECRET_KEY

eval "$MINIO_RUN"
