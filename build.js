'use strict';

const { join } = require('path');
const fs = require('fs');

const source = join(__dirname, 'native/index.node');
const dest = join(__dirname, `pre-built/${process.platform}_${process.versions.modules}.node`);

const copyFileSync = fs.copyFileSync || ((source, dest) => fs.writeFileSync(dest, fs.readFileSync(source)));

copyFileSync(source, dest);
process.stdout.write('\n\n\x1b[32mSuccessfully built native benchpress-rs addon.\n' +
  'Please contribute the new file to the repository to improve the experience of others.\n\n\x1b[39m');
