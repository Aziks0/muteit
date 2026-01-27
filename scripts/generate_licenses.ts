// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import assert from 'node:assert/strict';
import path from 'node:path';

// This is a weird hack to lock the version of nunjucks while still getting its
// types. The last version is from 2023 so we should be fine type-wise.
// biome-ignore lint/suspicious/noTsIgnore: ts doesn't know about bun
// @ts-ignore
const nunjucks: typeof import('nunjucks') = await import('nunjucks@3.2.4');

import { SetCwdWorkspace, TempDirectory } from './utils.module';

import ccBy from '../LICENSES/CC-BY-NC-SA-4.0.txt';
import gpl from '../LICENSES/GPL-3.0-or-later.txt';
import ofl from '../LICENSES/OFL-1.1.txt';

// Just what we need. There are way more objects in reality.
interface License {
  name: string;
  id: string;
  text: string;
  // biome-ignore lint/style/useNamingConvention: we didn't chose the name.
  used_by: {
    crate: {
      name: string;
    };
  }[];
}

type Licenses = License[];

const trimNewLines = (licenses: Licenses) => {
  licenses.forEach((license) => {
    if (license.text.length === 0) return;

    let first = 0;
    for (; first < license.text.length; first++) {
      const char = license.text[first];
      if (char !== '\n' && char !== '\r') break;
    }

    let last = license.text.length;
    for (; last >= 0; last--) {
      const char = license.text[last - 1];
      if (char !== '\n' && char !== '\r') break;
    }

    license.text = license.text.substring(first, last);
  });
};

const removeOurselves = (licenses: Licenses) => {
  const gplIndex = licenses.findIndex(
    (license) => license.id === 'GPL-3.0-or-later',
  );
  assert.notEqual(
    gplIndex,
    -1,
    '`GPL-3.0-or-later` license is not in the list.',
  );
  const muteitIndex = licenses[gplIndex].used_by.findIndex(
    (usedBy) => usedBy.crate.name === 'muteit',
  );
  assert.notEqual(
    muteitIndex,
    -1,
    '`muteit` is not in the list of `GPL-3.0-or-later` "used by".',
  );

  // Remove the license entirely if we are the only one using it.
  if (licenses[gplIndex].used_by.length === 1) {
    licenses.splice(gplIndex, 1);
  } else {
    licenses[gplIndex].used_by.splice(muteitIndex, 1);
  }
};

const checkCargoAboutVersion = async () => {
  const res = await $`cargo about --version`.text();
  const version = res.replace('cargo-about', '').trim();

  if (!Bun.semver.satisfies(version, '0.x.x'))
    throw Error(`cargo-about version is ${version}`);
};

/**
 * Generate license text for `muteit` and all its dependencies.
 *
 * @returns The license text.
 */
const generate = async (): Promise<string> => {
  using temp = new TempDirectory();
  using _wd = await SetCwdWorkspace.new();

  // Different versions might give different outputs.
  checkCargoAboutVersion();

  // `cargo-about` disallow stdout for powershell, so we have to use a file
  // instead.
  const cargoAboutLicensesPath = path.join(temp.path, 'licenses.json');
  const { exitCode } = await $`
    cargo about generate \
    -c crates/muteit/assets/bundle/about.toml \
    -o ${cargoAboutLicensesPath} \
    --format json \
    --fail
    `.nothrow();
  if (exitCode !== 0) throw Error('`cargo about` failed.');

  const licenseList = await Bun.file(cargoAboutLicensesPath).json();
  const licenses = licenseList.licenses;
  // We already have `muteit`'s license in the template.
  removeOurselves(licenses);
  // We don't want to trim whitespace. Only **newlines**. Otherwise the
  // `APACHE-2.0` headers (and maybe others) will be poorly formatted.
  trimNewLines(licenses);

  nunjucks.configure({ autoescape: false, throwOnUndefined: true });
  return nunjucks.render('crates/muteit/assets/bundle/licenses.md.jinja', {
    licenses: licenses,
    // biome-ignore-start lint/style/useNamingConvention: different convention
    // for jinja.
    GPL_3_0_OR_LATER_LICENSE_TEXT: gpl.trim(),
    OFL_1_1_LICENSE_TEXT: ofl.trim(),
    CC_BY_NC_SA_4_0_LICENSE_TEXT: ccBy.trim(),
    // biome-ignore-end lint/style/useNamingConvention: /
  });
};

export default generate;

if (import.meta.main) {
  SetCwdWorkspace.set();

  const outputDirectory = 'target';
  const outputPath = path.join(outputDirectory, 'LICENSES.txt');
  await $`mkdir -p ${outputDirectory}`;
  await $`rm -f ${outputPath}`;

  const licenses = await generate();
  Bun.write(outputPath, licenses);
}
