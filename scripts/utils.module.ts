// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

import { $ } from 'bun';

import fs from 'node:fs';
import path from 'node:path';

/**
 * Helper function to throw in ternary expression.
 *
 * @example const path = process.env.PATH ?? thrower('PATH not set.');
 *
 * @param message The thrown error message.
 *
 * @throws obviously.
 */
export const thrower = (message: string) => {
  throw Error(message);
};

/**
 * @see {@link new} for constructing it.
 * @see {@link set}
 */
export class SetCwdWorkspace implements Disposable {
  // @ts-expect-error: We're using `new` instead of `constructor`.
  #previousCwd: string;

  // We have to use an async function, which constructors cannot await. So we
  // use a static method as a constructor instead.
  private constructor() {}

  /**
   * Set the working directory to the workspace root directory.
   *
   * @description Set the working directory to the workspace root directory.
   * When the object gets popped off the stack, the previous working directory
   * (i.e. the one when {@link new} was called) is set back.
   *
   * Note that this method is async, and should be awaited.
   *
   * @example using _wd = await SetCwdWorkspace.new();
   *
   * @returns The constructed object.
   */
  static async new(): Promise<SetCwdWorkspace> {
    const this_ = new SetCwdWorkspace();

    const pwd = await $`pwd`.text();
    this_.#previousCwd = pwd.trimEnd();

    SetCwdWorkspace.set();

    return this_;
  }

  /**
   * Set the working directory to the workspace root directory.
   */
  static set() {
    const scriptDirectory = import.meta.dir;
    const workspaceDirectory = path.dirname(scriptDirectory);
    $.cwd(workspaceDirectory);
  }

  [Symbol.dispose]() {
    $.cwd(this.#previousCwd);
  }
}

export class TempDirectory implements Disposable {
  readonly path: string;

  /**
   * Create a temp directory.
   *
   * @description Create a temp directory that will be automatically removed
   * when the object gets popped off the stack.
   *
   * @example using temp = new TempDirectory();
   *
   * @throws Will throw an error if the `TEMP` env doesn't exist.
   * @throws Will throw an error if the temporary directory couldn't be created.
   */
  constructor() {
    const tempDirectory = process.env.TEMP ?? thrower('`TEMP` env is not set.');
    this.path = path.join(tempDirectory, Bun.randomUUIDv7());
    fs.mkdirSync(this.path, { recursive: false });
  }

  [Symbol.dispose]() {
    fs.rmSync(this.path, { recursive: true, force: true });
  }
}
