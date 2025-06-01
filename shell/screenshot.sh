#!/usr/bin/env bash

MENU_SELECTOR="dmenu"

# Acceptable values: file, clipboard
TARGET_LOCATION="$1"

# If second param is given, everything will be captured


function prompt_area {
  # Options for type of screenshot
  read -r -d '\n' options <<EOF
Selection
Focused Window
Focused Monitor
All Monitors
EOF

  # Choice of area
  selected=$(echo "$options" | $MENU_SELECTOR -nb '#2f343f' -nf '#f3f4f5' -sb '#f0544c' -sf '#1f222d' -fn '-*-*-medium-r-normal-*-*-*-*-*-*-100-*-*' -i)

  # Turn area into command parameters
  case "$selected" in
    "Selection")
      echo "Selection"
      cmd='grim -g "$(slurp)"'
      ;;
    "Focused Monitor")
      echo "Focused Monitor"
      cmd=$'grim -o "$(swaymsg -t get_outputs | jq -r \'.[] | select(.focused) | .name\')"'
      ;;
    "Focused Window")
      echo "Focused Window"
      cmd=$'grim -g "$(swaymsg -t get_tree | jq -j \'.. | select(.type?) | select(.focused).rect | "\(.x),\(.y) \(.width)x\(.height)"\')"'
      ;;
    "All Monitors")
      echo "All Monitors"
      cmd='grim'
      ;;
    *)
      echo "Error: Unknown selection: $selected !"
      exit 1
  esac
}

# Output type from parameter
if [ "$TARGET_LOCATION" == "file" ]; then
  mkdir -p "$HOME/screenshots"
  file="$HOME/screenshots/$(date +'%s.png')"
elif [ "$TARGET_LOCATION" == "clipboard" ]; then
  pipe='- | wl-copy'
else
  echo "Unknown Parameter"
fi

# If there's a second parameter, grab the entire screen regardless
if [ "$#" -eq 2 ]; then
    cmd="grim"
else
    prompt_area
fi

echo "$cmd $file $pipe"
eval "$cmd $file $pipe"

# vim: ts=4:et:sw=4:sts=4:noai
