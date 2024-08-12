#!/bin/bash

set -e

SCRIPT="true"
EXTRA_ARGS=()

# Login to docker registry
if [ ! -z $DOCKER_RUN_USERNAME ]; then
    echo "$DOCKER_RUN_PASSWORD" | docker login "$DOCKER_RUN_REGISTRY" -u "$DOCKER_RUN_USERNAME" --password-stdin
fi

# Join the specified docker network
if [ ! -z $DOCKER_RUN_DOCKER_NETWORK ]; then
    DOCKER_RUN_OPTIONS="$DOCKER_RUN_OPTIONS --network $DOCKER_RUN_DOCKER_NETWORK"
fi

# Use the specified user
if [ ! -z $DOCKER_RUN_USER ]; then
    USER_DIR="$RUNNER_TEMP/_home_$DOCKER_RUN_USER"

    USER_ID=$(id -u)
    GROUP_ID=$(id -g)

    mkdir -p "$USER_DIR"
    chown -R $USER_ID:$GROUP_ID "$USER_DIR"

    EXTRA_ARGS=( \
        --user $USER_ID:$GROUP_ID \
        -v "$USER_DIR":"/home/$DOCKER_RUN_USER" \
    )

    for GROUP in $(id -G); do
        EXTRA_ARGS+=(--group-add "$GROUP")
    done

    SCRIPT="$SCRIPT; export HOME=/home/$DOCKER_RUN_USER"
fi

# Setup known hosts
if [ "$DOCKER_RUN_SETUP_KNOWN_HOSTS" == "true" ]; then
    SCRIPT="$SCRIPT; mkdir -p ~/.ssh; ssh-keyscan -H github.com >> ~/.ssh/known_hosts"
fi

# Parse passed keys
if [ ! -z "$DOCKER_RUN_SSH_KEYS" ]; then
    SCRIPT="$SCRIPT; eval \"\$(ssh-agent -s)\""
    while IFS= read -r KEY; do
        if [ ! -z "$KEY" ]; then
            SCRIPT="$SCRIPT; echo \"$KEY\" | base64 -d | ssh-add -"
        fi
    done <<< "$DOCKER_RUN_SSH_KEYS"
fi

# Parse the passed volumes
while IFS= read -r VOLUME; do
    IFS=":" read -ra PARTS <<< "$VOLUME"

    if [ "${#PARTS[@]}" != "2" ]; then
        continue;
    fi

    VOLUME_NAME="$HOSTNAME-${PARTS[0]}"
    VOLUME_PATH="${PARTS[1]}"

    if ! docker volume ls --format '{{.Name}}' | grep -q "^${VOLUME_NAME}$"; then
        echo "Create docker volume: $VOLUME_NAME"

        docker volume create "$VOLUME_NAME"
    else
        echo "Reuse existing docker volume: $VOLUME_NAME"
    fi

    EXTRA_ARGS+=(-v "$VOLUME_NAME":"$VOLUME_PATH")
done <<< "$DOCKER_RUN_VOLUMES"

# Set environment variables
for ENV in $(export -p | cut -d' ' -f3 | cut -d'=' -f1 | grep -vE '^(OLDPWD|PATH|PWD|SHL|HOME|HOSTNAME|INPUT_.*|DOCKER_RUN_.*)$'); do
    EXTRA_ARGS+=(-e "$ENV")
done

# Bring up the container and execute the requested command
SCRIPT="$SCRIPT; $DOCKER_RUN_RUN"

exec docker run \
    --rm \
    ${EXTRA_ARGS[@]} \
    -v "/var/run/docker.sock":"/var/run/docker.sock" \
    -v "$GITHUB_WORKSPACE":"/github/workspace" \
    --workdir /github/workspace \
    $DOCKER_RUN_OPTIONS \
    --entrypoint="$DOCKER_RUN_SHELL" \
    "$DOCKER_RUN_IMAGE" \
        -c "$SCRIPT"
