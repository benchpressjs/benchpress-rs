#!/bin/bash
set -eo pipefail
IFS=$'\n\t'

echo "Adding binary"

node scripts/copy-binary
git add pre-built/*.node
git commit -m "Updated binary module [skip ci]"
git pull --rebase

# retry if fails, probably because race condition
NEXT_WAIT_TIME=0
until ( git push && echo "Successfully added binary" ) || [[ $NEXT_WAIT_TIME = 4 ]]; do
  sleep $(( NEXT_WAIT_TIME++ ))
  git pull --rebase
done
