IF "%APPVEYOR_PULL_REQUEST_NUMBER%"=="" (
  set publish_binary=true

  git config --global user.email "p.jaszkow@gmail.com"
  git config --global user.name "Appveyor CI"

  REM get base branch 'master_builds' to 'master'
  set base_branch=%APPVEYOR_REPO_BRANCH:_builds=%
)

rustup install %CLIPPY_TOOLCHAIN%
rustup component add clippy --toolchain=%CLIPPY_TOOLCHAIN%
