// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

// Get the focus off of the button, otherwise the user might interact with it
// when binding the hotkey.
document.getElementById('bind-button').blur();

const controller = new AbortController();

document.addEventListener(
  'keydown',
  async (event) => {
    event.preventDefault();

    if (event.repeat) return;

    await dioxus.send({
      keyPress: {
        ctrl: event.ctrlKey,
        shift: event.shiftKey,
        alt: event.altKey,
        meta: event.metaKey,
        code: event.code,
        key: event.key,
      },
    });
  },
  { signal: controller.signal },
);

document.addEventListener(
  'keyup',
  async (_event) => {
    controller.abort();

    await dioxus.send('finished');
  },
  { signal: controller.signal },
);
