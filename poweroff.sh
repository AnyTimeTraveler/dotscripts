#!/usr/bin/env bash

read -r -d '\n' options <<EOF
Nothing
Sleep
Shutdown
Reboot
Hibernate
Logout
EOF

#selected=$(echo "$options" | aphorme --select-from-stdin)
selected=$(echo "$options" | anyrun --plugins libstdin.so)
#selected=$(echo "$options" | kickoff --from-stdin --stdout)

case "$selected" in
  Nothing)
    echo "Doing nothing :)"
    ;;
  Sleep)
    echo "Sleeping..."
    systemctl suspend
    ;;
  Shutdown)
    echo "Shutting down..."
    systemctl poweroff
    ;;
  Reboot)
    echo "Rebooting..."
    systemctl reboot
    ;;
  Hibernate)
    echo "Hibernating..."
    systemctl hibernate
    ;;
  Logout)
    echo "Logging out..."
    swaymsg exit
    ;;
  *)
    echo "Error: Unknown selection: $selected !"
    exit 1
esac

# vim: ts=2:et:sw=2:sts=2:noai
