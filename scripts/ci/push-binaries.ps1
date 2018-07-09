if ($env:publish_binary -eq "true" -and $env:final_build -eq "true") {
  echo "Pushing updated binaries"

  $remote = "https://$($env:gh_token):@github.com/benchpressjs/benchpress-rs.git"

  git fetch

  # add built binaries to the main branch
  git checkout $env:base_branch
  git checkout "origin/$env:APPVEYOR_REPO_BRANCH" pre-built
  git add --all pre-built
  git commit -m "Updated Windows binary modules"
  git push -q $remote
}