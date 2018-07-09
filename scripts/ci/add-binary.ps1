if ($env:publish_binary -eq "true") {
  echo "Adding binary"

  git fetch
  git checkout -q $env:APPVEYOR_REPO_BRANCH
  git pull
  node scripts/copy-binary
  git add --all pre-built
  git commit -m "Updated binary module [skip ci]"
  
  # using the full URL here instead of a credentials file
  $remote = "https://$($env:gh_token):@github.com/benchpressjs/benchpress-rs.git"

  # retry if fails, probably because race condition
  foreach ($next_wait_time in 0..4) {
    git push -q $remote

    if ($?) {
      echo "Successfully published binary"
      exit 0
    } else {
      echo "Push failed, waiting $next_wait_time sec and pulling"
      
      Start-Sleep $next_wait_time
      git pull --rebase
    }
  }

  echo "Failed to publish binary after 5 attempts"
  exit 1
}