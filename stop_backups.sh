#!/usr/bin/env sh
sudo systemctl stop restic-backups-localbackup.timer
sudo systemctl stop restic-backups-localbackup.service
sudo systemctl stop restic-backups-remotebackup.timer
sudo systemctl stop restic-backups-remotebackup.service
