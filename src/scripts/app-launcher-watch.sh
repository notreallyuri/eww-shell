#!/bin/bash

SEARCH=""

while true; do
  NEW_SEARCH=$(eww get search_text 2>/dev/null || echo "")

  if [ "$NEW_SEARCH" != "${SEARCH}" ]; then
    SEARCH="$NEW_SEARCH"
    ./src/plugins/app-launcher "${SEARCH}"
  fi

  sleep 0.1
done
