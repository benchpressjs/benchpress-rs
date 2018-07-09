if [[ "${PUBLISH_BINARY}" = "true" ]]; then
  echo "Adding binary"

  git fetch
  git checkout "${TRAVIS_BRANCH}"
  git pull
  node scripts/copy-binary
  git add --all pre-built
  git commit -m "Updated binary module [skip ci]"

  # retry if fails, probably because race condition
  NEXT_WAIT_TIME=0
  until ( git push && echo "Successfully published binary" ) || [[ $NEXT_WAIT_TIME = 4 ]]; do
    git pull --rebase
    sleep $(( NEXT_WAIT_TIME++ ))
  done
fi