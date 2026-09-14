// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import env from '../../crates/muteit/env.toml';

for (const [key, value] of Object.entries(env)) {
  const variable = `${key}=${value}`;
  await $`echo ${variable}`;
}
