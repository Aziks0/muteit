// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import { SetCwdWorkspace } from './utils.module';

using _wd = await SetCwdWorkspace.new();

$.nothrow();

const rustFlags = ['--cfg loom', process.env.RUSTFLAGS]
  .filter(Boolean)
  .join(' ');

// biome-ignore lint/style/useNamingConvention: env
$.env({ RUSTFLAGS: rustFlags });

const args = Bun.argv.slice(2).join(' ');
await $`cargo test --color "always" --release ${{ raw: args }}`;
