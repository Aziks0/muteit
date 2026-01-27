// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import path from 'node:path';

// This is a weird hack to lock the version of nunjucks while still getting its
// types. The last version is from 2023 so we should be fine type-wise.
// biome-ignore lint/suspicious/noTsIgnore: ts doesn't know about bun
// @ts-ignore
const nunjucks: typeof import('nunjucks') = await import('nunjucks@3.2.4');

import generateLicenses from './generate_licenses';
import { SetCwdWorkspace } from './utils.module';

import cargo from '../crates/muteit/Cargo.toml';
import env from '../crates/muteit/env.toml';

using _wd = await SetCwdWorkspace.new();

nunjucks.configure({ autoescape: false, throwOnUndefined: true });

const distDirectory = 'dist';
const filesDirectory = 'target/release/bundle';
/// Directory for files to add to the archive.
const archiveDirectory = path.join(filesDirectory, 'archive');
const distName = `${env.NAME}-${cargo.package.version}-x64`;

console.log('Cleaning up files...');
await $`rm -rf ${distDirectory} ${filesDirectory}`;
await $`mkdir -p ${distDirectory} ${archiveDirectory}`;

console.log('Building binary...');
const executablePath = path.join(archiveDirectory, `${env.NAME}.exe`);
await $`cargo build -F bundle --release`;
await $`mv target/release/muteit.exe ${executablePath}`;

console.log('Generating licenses...');
const licenseFilePath = path.join(archiveDirectory, 'LICENSES.md');
const licenses = await generateLicenses();
await Bun.write(licenseFilePath, licenses);

console.log('Generating setup...');
const setupFilePath = path.join(filesDirectory, 'setup.iss');
const setup = nunjucks.render('crates/muteit/assets/bundle/setup.iss.jinja', {
  ...env,
  // biome-ignore-start lint/style/useNamingConvention: different convention
  // for envs
  VERSION: cargo.package.version,
  ICON_PATH: path.resolve('crates/muteit/assets/muteit_icon.ico'),
  EXE_PATH: path.resolve(executablePath),
  LICENSE_PATH: path.resolve(licenseFilePath),
  // biome-ignore-end lint/style/useNamingConvention: /
});
await Bun.write(setupFilePath, setup);

console.log('Compiling setup...');
const setupFilename = `${distName}-setup`;
const isccOutput =
  $`iscc /O${distDirectory} /F${setupFilename} ${setupFilePath}`.lines();
try {
  for await (const line of isccOutput) {
    if (line.startsWith('Warning:')) {
      console.log(`  ${line}`);
    }
  }
} catch (error) {
  if (!(error instanceof $.ShellError)) {
    throw error;
  }

  console.log(error.stderr.toString().trim());
  throw Error('Setup compilation failed.');
}

console.log('Creating archive...');
const archivePath = path.join(distDirectory, `${distName}.zip`);
const filesGlob = `./${archiveDirectory}/*`;
await $`7z a -mx9 ${archivePath} ${filesGlob}`.quiet();

console.log('Success');
