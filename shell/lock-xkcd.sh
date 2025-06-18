#!/usr/bin/env bash

source $(which lock-common.sh)

cd ~
python3 ~/.scripts/xkcd.py
$lock -i ~/xkcd.png --show-failed-attempts
rm ~/xkcd.png

# vim: ts=2:et:sw=2:sts=2:noai
