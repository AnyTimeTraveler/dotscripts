#!/usr/bin/env bash

read -r -d '\n' options <<EOF
0
10
20
30
40
50
60
70
80
90
100
EOF

#selected=$(echo "$options" | aphorme --select-from-stdin)
selected=$(echo "$options" | anyrun --plugins libstdin.so)
#selected=$(echo "$options" | kickoff --from-stdin --stdout)

brightness.sh $selected

# vim: ts=2:et:sw=2:sts=2:noai
