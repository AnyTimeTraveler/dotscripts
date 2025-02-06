#!/usr/bin/env bash

if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
  export screenshotter="grim"
  export lock="swaylock"
else
  export screenshotter="scrot"
  export lock="i3lock"
fi

# vim: ts=2:et:sw=2:sts=2:noai
