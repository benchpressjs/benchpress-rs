const { execSync: exec } = require('child_process');

const branchName = exec('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();

try {
  try {
    exec(`git checkout ${branchName}_builds`);
  } catch (e) {
    exec(`git checkout -b ${branchName}_builds`);
  }
  exec(`git reset --hard ${branchName}`);
  try {
    exec('git rm pre-built/*.node');
  } catch (e) {}
  exec('git commit --allow-empty -m "Build new binaries [publish binary]"');
  exec(`git push -f -u origin ${branchName}_builds`);
} catch (e) {
  console.error(e);
} finally {
  exec(`git checkout ${branchName}`);
}
