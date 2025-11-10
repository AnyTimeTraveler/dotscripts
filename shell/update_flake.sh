#!/usr/bin/env zsh

# This script is meant to be executed by a recursive search algorithm on every found flake.nix file
# It updates the flake and then waits 5 minutes to prevent rate-limiting

echo "Updating $1"

(cd $(dirname $1); nix flake update)

sleep 300
