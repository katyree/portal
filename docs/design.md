# Portal implementation shape

The fixed catalog is the base design. Five known launchers do not need an adapter trait or dynamic plugin registry. Independent design sketches and a cross-judge favored a catalog module separated from rendering.

`catalog.rs` owns discovery, running-state checks, launching, and saved executable overrides. `ui.rs` owns the responsive dark Studio card grid, Settings list, and GPUI events. `search_input.rs` uses GPUI's standard text input example for clipboard, selection, Unicode, and input-method support.

The alternate adapter design contributed the distinction between executable launches and Xbox protocol activation. Arbitrary command strings and user-entered protocols are excluded. Launching rechecks the installation and uses an executable path directly, without a command shell.

Settings use temporary-file replacement. Invalid settings stop loading and remain intact. A generation number prevents an older background scan from replacing the result of a newer configuration. Process scans run outside the UI thread.

The initial verification covers settings round trips, rejection of non-executable overrides, preservation of corrupt settings, a Windows build, and the live launcher scan. See the repository README for commands.
