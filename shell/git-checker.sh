#!/usr/bin/env bash

declare -a outdated_repos

function echoerr {
  echo "$@" 1>&2
}

function check_repo {
  if [ ! -d "$1" ]; then
    return
  fi
  if [ -d "$1/.git" ]; then
    changed_files="$(git --work-tree="$1" status --porcelain | wc -l)"
    if [ "$changed_files" -gt 0 ]; then
      echoerr "Has changes: $1"
      outdated_repos+=( "$1" )
    else
      echoerr "Up to date:  $1"
    fi
  fi
}

check_repo "$HOME"

for folder in ${HOME}/*; do
  check_repo "$folder"
  for subfolder in ${folder}/*; do
    check_repo "$subfolder"
  done
done

# for repo in ${outdated_repos[*]}; do
#   echo "Out: $repo"
# done

whip=(--checklist \"Projects that have uncommited files\" 20 78 15)

whip_opts="$(for repo in ${outdated_repos[*]}; do printf ' %s %s off' "$(basename $repo)" "$repo"; done)"

do_commit=$(whiptail --checklist "Projects that have uncommited files" 45 150 38${whip_opts} 3>&1 1>&2 2>&3 | )

for repo in $do_commit; do
  echo "Processing: $repo"
  git --work-tree="$repo" add -A
  git --work-tree="$repo" commit --no-signoff -m "Automatic commit: Uncomitted files on computer shutdown"
  for remote in $(git --work-tree="$repo" remote); do
    git push -u $remote $(git rev-parse --abbrev-ref HEAD)
  done
done

# vim: ts=2:et:sw=2:sts=2:noai
