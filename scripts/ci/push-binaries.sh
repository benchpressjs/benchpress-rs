if [[ "${PUBLISH_BINARY}" = "true" ]]; then
  echo "Pushing updated binaries"

  git fetch

  # add built binaries to the main branch
  git checkout "${BASE_BRANCH}"
  git checkout "origin/${TRAVIS_BRANCH}" pre-built
  git add --all pre-built
  git commit -m "Updated linux binary modules"
  git push
fi