# portal

A minimal Windows game launcher dashboard built with Rust and GPUI. The dark Studio interface lists Amazon Games, Battle.net, Clone Hero Launcher, EA app, Epic Games, Game Jolt Client, GOG GALAXY, Hytale Launcher, IndieGala Client, itch, Microsoft Store, Riot Client, Rockstar Games, Steam, Ubisoft Connect, Xbox, and YARC Launcher alphabetically. Settings uses the same order.

## Install Portal

Run `portal-setup.exe`. Portal installs for your Windows account in `%LOCALAPPDATA%\Programs\Portal`, adds Start-menu shortcuts, and registers under **Settings > Apps > Installed apps**. You do not need administrator rights, Rust, or a separate Visual C++ runtime installer.

The installer requires Windows 10 version 2004 or later, or Windows 11, on a system that can run x64 applications. This prototype installer is unsigned.

To uninstall, close Portal and select **Uninstall** in Windows Settings, or use the **Uninstall Portal** Start-menu shortcut. The installer places `unins000.exe` beside `portal.exe`. The uninstaller removes installed files, shortcuts, and the Installed Apps entry. It preserves `%LOCALAPPDATA%\Portal\settings.json` and files you add to the installation directory.

Running the installer again updates the same installation. It uses a stable application ID and does not create a second uninstall entry.

## Run from source

Install stable Rust and the Visual Studio C++ build tools with the Windows SDK, then run:

```powershell
cargo run --locked
```

The dashboard arranges launchers in a responsive card grid, from one column in a narrow window to five on a wide screen. Content stays centered within a maximum width of 1,600 logical pixels. Portal opens at 1,000 × 700 logical pixels.

Click **Launch ↗** or **Open ↗** to open an automatically detected launcher. Each card shows **Ready to launch** or **Running**, with a green dot beside **Running**. Launchers that Portal cannot find show **Not installed** without a launch button. Use Search to filter the grid and the refresh button to scan again.

Turn on **Installed only** beside Search to hide launchers that Portal cannot find. The filter combines with Search and leaves integration settings unchanged. It resets to showing all enabled launchers when you reopen Portal.

Portal checks registered installation paths and common Windows locations, including Amazon Games under LocalAppData, Rockstar's `LauncherPatcher.exe`, and Battle.net's stable launcher executable. Xbox is discovered through its current-user Windows package and activated using its app ID, with the registered `xbox:` and `ms-xbox:` protocols as fallbacks. Custom executable paths are saved in `%LOCALAPPDATA%\Portal\settings.json`. Portal refreshes status every five seconds. It does not collect game libraries or sign into platform accounts.

Open **Settings** to enable or disable each launcher integration. Disabled launchers disappear from the dashboard and are excluded from discovery, status tracking, and launching. Choices persist across restarts. Turning an integration off preserves its custom executable path and leaves the launcher installed and running.

GOG GALAXY, Riot Client, Game Jolt Client, and IndieGala Client use registered installation locations and known executable paths. Riot's client-installation manifest also supports custom locations. itch uses its stable `itch-setup.exe` launcher. Microsoft Store uses its Windows package app ID, with the registered `ms-windows-store:` protocol as a fallback. Existing custom executable paths remain supported; unavailable saved paths fall back to automatic discovery.

Hytale Launcher, Clone Hero Launcher, and YARC Launcher also use registered installation locations and common Windows paths. Their executable names are `hytale-launcher.exe`, `ch_launcher.exe`, and `YARC Launcher.exe`. YARC Launcher manages Yet Another Rhythm Game, or YARG; Portal opens the launcher rather than the game.

## Check the build

```powershell
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

The debug binary also supports these commands:

```powershell
.\target\debug\portal.exe --scan
.\target\debug\portal.exe --launch steam
.\target\debug\portal.exe --settings
```

Launcher names are `amazon`, `battledotnet`, `clonehero`, `ea`, `epicgames`, `gamejolt`, `goggalaxy`, `hytale`, `indiegala`, `itch`, `microsoftstore`, `riot`, `rockstargames`, `steam`, `ubisoft`, `xbox`, and `yarc`.

Set `PORTAL_SETTINGS_DIR` to use an isolated settings directory for testing. Saved paths stay outside the repository. The packaged executable embeds its launcher icons and does not need the assets directory beside it.

## Portal branding

Portal uses the Nexus mark in its header, Windows executable, installer, and shortcuts. `assets/portal.svg` is the editable source. `assets/portal.ico` contains 17 sizes from 16 through 256 pixels; `assets/icons` contains each PNG size, and `assets/portal.png` is the 512-pixel UI image.

Generated assets are checked in, so normal Rust builds do not need Node.js. To regenerate them after editing the SVG:

```powershell
npm install --prefix .tools/branding --no-save --no-package-lock @resvg/resvg-js@2.6.2
node scripts/generate-branding.cjs
```

## Build the installer

```powershell
./scripts/build-installer.ps1
```

This builds the release executable and writes `dist/portal-setup.exe` and its SHA-256 file. The script downloads the pinned, signed Inno Setup 6.7.1 compiler, verifies its hash, and extracts it in portable mode under `.tools`. To use a compiler you already have, pass `-CompilerPath 'C:\path\to\ISCC.exe'`.

The `portal-setup.exe` filename avoids Windows' special compatibility behavior for executables named exactly `setup.exe`.

To verify installation and uninstallation in isolated folders:

```powershell
./scripts/test-installer.ps1
```

The check refuses to run if Portal is already installed for the current account. It verifies the copied executable, launcher scanning, shortcuts, Installed Apps registration, reinstallation, uninstall removal, and preservation of settings and unrelated files. Logs are retained under `evidence`.

## Initial scope

This prototype targets Windows. Launch detection and process names vary across launcher updates. A process match reports running state; it does not confirm that a launcher is signed in or ready to play.

## Credits

The text input is adapted from [GPUI 0.2.2's input example](https://docs.rs/crate/gpui/0.2.2/source/examples/input.rs), under Apache-2.0. The launcher marks come from [Simple Icons](https://github.com/simple-icons/simple-icons). Xbox, GOG, Riot, itch, Game Jolt, and Microsoft Store marks are from Simple Icons 12.4.0. IndieGala, Hytale, Clone Hero, and YARC use letter badges. Brand marks belong to their respective owners.
