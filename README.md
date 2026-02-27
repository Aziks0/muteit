<!--
  SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>

  SPDX-License-Identifier: CC0-1.0
-->

<!-- markdownlint-disable MD059 -->

<!-- markdownlint-disable-next-line MD026 -->
# muteit!

Add a mute icon to your system tray.

<!-- TODO: upload demo using Github -->

## Features

- Get a quick look to know if your mic is muted with the tray icon
- Click on the icon to toggle your mic's state
- Use a hotkey to toggle your mic's state

## Requirements

- Windows 10+
- WebView2 _(pre-installed on Windows, but the setup installs it if needed)_

## Download

Head over to the [releases](https://github.com/Aziks0/muteit/releases/latest) page.

## Settings

Right click on the tray icon and click on `Settings` to open the settings window.

> [!NOTE]
> The settings file is located at `%LOCALAPPDATA/muteit!/config.toml`.
>
> If your using the single executable, a directory is still created at `%LOCALAPPDATA%/muteit!`.

## Licensing

This repository contains sources available under a multitude of open source licenses.

`muteit!`, as a whole, is licensed under the GNU General Public License version 3 or later. A copy of the licensing
terms is available [here](LICENSES/GPL-3.0-or-later.txt).

Being [REUSE-compliant](https://reuse.software/), every file, if possible, contains a license header with information on
its copyrights. You can get a Software Bill Of Materials by installing the [REUSE
tool](https://codeberg.org/fsfe/reuse-tool.git), cloning this repository, and running `reuse spdx -o sbom.spdx`.

Is made available to download, on the [release page](https://github.com/Aziks0/muteit/releases/latest) and for every
release, a `LICENSES.md` file containing all the licensing terms of the code used by this software.
