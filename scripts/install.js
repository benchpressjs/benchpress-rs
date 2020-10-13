'use strict';

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const dest = path.join(__dirname, '../index.node');

function build_module() {
  console.log('[benchpress] Building native module from source...');

  try {
    execFileSync(
      path.join(__dirname, '../node_modules/.bin/neon'),
      ['build', '--release'],
      { stdio: 'inherit' }
    );
  } catch (err) {
    console.error('[benchpress] FATAL: Fallback build failed. For more info, see https://github.com/benchpressjs/benchpressjs#manually-building-native-module');
    process.exit(1);
  }

  console.log('[benchpress] Copying newly built module into target location.');
  const source = path.join(__dirname, '../native/index.node');
  fs.copyFileSync(source, dest);
}

if (process.env.npm_config_build_from_source === 'true' || process.env.BENCHPRESS_FORCE_BUILD === 'true') {
  build_module();
  console.log('[benchpress] Successfully completed install step.');
  return;
}

const modulePath = path.join(__dirname, `../pre-built/${process.platform}_${process.versions.modules}.node`);

function prebuilt_module_exists() {
  try {
    fs.statSync(modulePath);
  } catch (err) {
    if (err.code === 'ENOENT') {
      return false;
    }
    throw err;
  }

  return true;
}

if (prebuilt_module_exists()) {
  console.log('[benchpress] Copying pre-built module into target location.');
  fs.copyFileSync(modulePath, dest);
} else {
  console.warn('[benchpress] No compatible pre-built native module found!', {
    platform: process.platform, module_version: process.versions.modules, node: process.versions.node
  });

  if (process.env.BENCHPRESS_SKIP_FALLBACK === 'true') {
    console.warn('[benchpress] Env flag BENCHPRESS_SKIP_FALLBACK asserted!');
    console.error('[benchpress] FATAL: Skipping fallback build from source.');
    process.exit(1);
  }

  build_module();
}

console.log('[benchpress] Successfully completed install step.');
