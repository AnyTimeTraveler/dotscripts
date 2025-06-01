#!/usr/bin/env bash

source $HOME/.scripts/lock-common.sh

rawFile="$HOME/tempRawLockscreen.png"
editedFile="$HOME/tempBlurredLockscreen.png"

$screenshotter $rawFile
convert $rawFile -resize 10% -filter Box -resize 1000% $editedFile
rm $rawFile
$lock -i $editedFile --show-failed-attempts
rm $editedFile

# vim: ts=2:et:sw=2:sts=2:noai
