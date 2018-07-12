'use strict';

const { join } = require('path');
const { readFileSync, writeFileSync } = require('fs');

const rustVersions = [
  'stable',
  'beta',
  'nightly'
];

const nodeVersions = [
  '6',
  '8',
  '9',
  '10'
];

const indent = (str, n) => str.split('\n').map(x => `${' '.repeat(4)}${x}`).join('\n');

const testJobs = [];

rustVersions.forEach(rustVersion => nodeVersions.forEach((nodeVersion) => {
  testJobs.push({ rustVersion, nodeVersion });
}));

const buildJobs = nodeVersions.map(nodeVersion => ({ nodeVersion }));

const travisBase = readFileSync(join(__dirname, 'base.travis.yml'), 'utf8');
const avBase = readFileSync(join(__dirname, 'base.appveyor.yml'), 'utf8');

const av = `
${avBase}
${testJobs.map(({ rustVersion, nodeVersion }) => `
      - rust_channel: ${rustVersion}
        nodejs_version: ${nodeVersion}
`).join('')}
`.replace('${NODE_VERSIONS}', buildJobs.map(({ nodeVersion }) => `
      - nodejs_version: ${nodeVersion}
`.trimRight()).join(''));

const travis = `
${travisBase}
${testJobs.map(({ rustVersion, nodeVersion }) => `
    - stage: test
      rust: ${rustVersion}
      env: TRAVIS_NODE_VER=${nodeVersion}
`).join('')}


${buildJobs.map(({ nodeVersion }) => `
    - stage: build binaries
      rust: stable
      env: TRAVIS_NODE_VER=${nodeVersion}
      before_script: source ./scripts/ci/vars.sh
      after_success: source ./scripts/ci/add-binary.sh
`).join('')}
`;

writeFileSync(join(__dirname, '../.travis.yml'), travis);
writeFileSync(join(__dirname, '../appveyor.yml'), av);