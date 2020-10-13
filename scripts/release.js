'use strict';

const { execSync: exec } = require('child_process');
const assert = require('assert');

const branchName = exec('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();

try {
  exec('git fetch');
  const status = exec(`git status -uno`, { encoding: 'utf8' });
  assert(
    status.includes('nothing to commit'),
    'Working tree must be clean before release. Commit any changes then try again.'
  );
  assert(
    status.includes('branch is up to date'),
    'Must be synced with remote before running a release.'
  );
  exec(`git checkout -B ${branchName}_builds`);
  exec(`git reset --hard ${branchName}`);
  try {
    exec('git rm pre-built/*.node');
  } catch (e) {
    // ignore remove failures
  }
  exec('git commit --allow-empty -m "Build new binaries [publish binary]"');
  exec(`git push -f -u origin ${branchName}_builds`);
} catch (e) {
  // eslint-disable-next-line no-console
  console.error(e.stack);
} finally {
  exec(`git checkout ${branchName}`);
}
