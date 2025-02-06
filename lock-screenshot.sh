#!/usr/bin/env bash

source $HOME/.scripts/lock-common.sh

file="$HOME/tempLockscreen.png"

$screenshotter $file
$lock -i $file --show-failed-attempts
rm $file

# vim: ts=2:et:sw=2:sts=2:noai
