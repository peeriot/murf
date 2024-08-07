#!/bin/bash

# Login to docker registry
if [ ! -z $INPUT_USERNAME ]; then
    echo "$INPUT_PASSWORD" | docker login "$INPUT_REGISTRY" -u "$INPUT_USERNAME" --password-stdin
fi

# Join the specified docker network
if [ ! -z $INPUT_DOCKER_NETWORK ]; then
    INPUT_OPTIONS="$INPUT_OPTIONS --network $INPUT_DOCKER_NETWORK"
fi

# Use the specified user
if [ ! -z $INPUT_USER ]; then
    USER_DIR="$RUNNER_TEMP/_home_$INPUT_USER"

    mkdir -p "$USER_DIR"

    USER_ARGS=( \
        --user $(id -u):$(id -g) \
        -v "$USER_DIR":"/home/$INPUT_USER" \
    )
fi

# Parse the passed volumes
VOLUMES=()
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

    VOLUMES+=(-v "$VOLUME_NAME":"$VOLUME_PATH")
done <<< "$INPUT_VOLUMES"

# Bring up the container and execute the requested command
exec docker run \
    --rm \
    ${VOLUMES[@]} \
    ${USER_ARGS[@]} \
    -v "/var/run/docker.sock":"/var/run/docker.sock" \
    -v "$GITHUB_WORKSPACE":"/github/workspace" \
    --workdir /github/workspace \
    $INPUT_OPTIONS \
    --entrypoint="$INPUT_SHELL" \
    "$INPUT_IMAGE" \
        -c "${INPUT_RUN//$'\n'/;}"
