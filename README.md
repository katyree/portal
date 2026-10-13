# portal

A minimal Windows game launcher dashboard built with Rust and GPUI. The dark Studio interface puts Steam, Ubisoft Connect, Epic Games, Xbox, and EA in one list.

## Run Portal

Install stable Rust and the Visual Studio C++ build tools with the Windows SDK, then run:

```powershell
cargo run --locked
```

Click the arrow to open an installed launcher. A green dot means its process is running. Click the folder button or the plus button to select a launcher executable when automatic detection misses it. Use Search to filter the list and the refresh button to scan again.

Portal checks registered installation paths and common Windows locations. Xbox uses the registered `ms-xbox:` protocol. Custom executable paths are saved in `%LOCALAPPDATA%\Portal\settings.json`. Portal refreshes status every five seconds. It does not collect game libraries or sign into platform accounts.

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

Other launcher names are `ubisoft`, `epicgames`, `xbox`, and `ea`.

Set `PORTAL_SETTINGS_DIR` to use an isolated settings directory for testing. Saved paths stay outside the repository. The packaged executable embeds its launcher icons and does not need the assets directory beside it.

## Initial scope

This prototype targets Windows. Launch detection and process names vary across launcher updates. Use the executable picker for custom installation locations. A process match reports running state; it does not confirm that a launcher is signed in or ready to play.

## Credits

The text input is adapted from [GPUI 0.2.2's input example](https://docs.rs/crate/gpui/0.2.2/source/examples/input.rs), under Apache-2.0. The launcher marks come from [Simple Icons](https://github.com/simple-icons/simple-icons). Xbox's mark is from Simple Icons 12.4.0. Brand marks belong to their respective owners.
