'use strict';

const { join } = require('path');
const fs = require('fs');

const copyFileSync = fs.copyFileSync ||
  ((source, dest) => fs.writeFileSync(dest, fs.readFileSync(source)));

const source = join(__dirname, '../native/index.node');
const dest = join(__dirname, `../pre-built/${process.platform}_${process.versions.modules}.node`);

copyFileSync(source, dest);
