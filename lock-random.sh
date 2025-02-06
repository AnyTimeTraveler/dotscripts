#!/usr/bin/env bash

file=$(ls ~/.scripts/ | grep lock | grep -v common |grep -v random | sort -R | tail -n 1)
source $HOME/.scripts/$file

# vim: ts=2:et:sw=2:sts=2:noai
