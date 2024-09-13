#!/usr/bin/env bash

# Name of the screen session
SCREEN_NAME="kanshi"

# Get the current Git commit hash
GIT_HASH=$(git rev-parse HEAD)

# Directory to store the commit hash file
BUILD_INFO_DIR="./target"
BUILD_INFO_FILE="$BUILD_INFO_DIR/.build_hash"

# Function to build the project
build_project() {
    echo "Building the project..."
    cargo build --release
    echo "Storing commit hash..."
    echo "$GIT_HASH" > "$BUILD_INFO_FILE"
}

# Function to attach to the screen session
attach_screen() {
    echo "Attaching to the existing screen session..."
    screen -r "$SCREEN_NAME"
}

# Check if the target directory exists
if [ ! -d "$BUILD_INFO_DIR" ]; then
    mkdir -p "$BUILD_INFO_DIR"
fi

# Check if the project is already built with the current Git commit hash
if [ -f "$BUILD_INFO_FILE" ] && grep -q "$GIT_HASH" "$BUILD_INFO_FILE"; then
    echo "Project already built with the current commit: $GIT_HASH"
else
    build_project
fi

# Check if the screen session is already running
if screen -list | grep -q "$SCREEN_NAME"; then
    echo "Screen session $SCREEN_NAME is already running."
    attach_screen
else
    echo "Starting a new screen session for $SCREEN_NAME..."
    screen -dmS "$SCREEN_NAME" ./target/release/kanshi
    echo "New screen session $SCREEN_NAME started."
    attach_screen
fi
