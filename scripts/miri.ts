// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import { SetCwdWorkspace } from './utils.module';

using _wd = await SetCwdWorkspace.new();

$.nothrow();

const args = Bun.argv.slice(2).join(' ');
await $`cargo +nightly miri test --color "always" ${{ raw: args }}`;
