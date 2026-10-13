# portal

A minimal Windows game launcher dashboard built with Rust and GPUI. The dark Studio interface puts Steam, Ubisoft Connect, Epic Games, Xbox, EA, Rockstar Games, Battle.net, and Amazon Games in one list.

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

Click the arrow to open an installed launcher. The adjacent label says **Running** or **Launch**, with a green dot beside **Running**. Click the folder button or the plus button to select a launcher executable when automatic detection misses it. Use Search to filter the list and the refresh button to scan again.

Portal checks registered installation paths and common Windows locations, including Amazon Games under LocalAppData, Rockstar's `LauncherPatcher.exe`, and Battle.net's stable launcher executable. Xbox is discovered through its current-user Windows package and activated using its app ID, with the registered `xbox:` and `ms-xbox:` protocols as fallbacks. Custom executable paths are saved in `%LOCALAPPDATA%\Portal\settings.json`. Portal refreshes status every five seconds. It does not collect game libraries or sign into platform accounts.

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
```

Other launcher names are `ubisoft`, `epicgames`, `xbox`, `ea`, `rockstargames`, `battledotnet`, and `amazon`.

Set `PORTAL_SETTINGS_DIR` to use an isolated settings directory for testing. Saved paths stay outside the repository. The packaged executable embeds its launcher icons and does not need the assets directory beside it.

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

This prototype targets Windows. Launch detection and process names vary across launcher updates. Use the executable picker for custom installation locations. A process match reports running state; it does not confirm that a launcher is signed in or ready to play.

## Credits

The text input is adapted from [GPUI 0.2.2's input example](https://docs.rs/crate/gpui/0.2.2/source/examples/input.rs), under Apache-2.0. The launcher marks come from [Simple Icons](https://github.com/simple-icons/simple-icons). Xbox's mark is from Simple Icons 12.4.0. Brand marks belong to their respective owners.
