PUBLISH_BINARY=false
# only publish binary if commit tells us to
if [[ "$TRAVIS_COMMIT_MESSAGE" = *"[publish binary]"* ]]; then
  PUBLISH_BINARY=true

  git config --global user.email "p.jaszkow@gmail.com"
  git config --global user.name "Travis CI"
  git config credential.helper "store --file=.git/credentials"
  echo "https://${GH_TOKEN}:@github.com" > .git/credentials

  # get base branch master_builds->master
  BASE_BRANCH="${TRAVIS_BRANCH%_builds}"
fi

rustup install $CLIPPY_TOOLCHAIN
rustup component add clippy-preview --toolchain=$CLIPPY_TOOLCHAIN
