# DesktopIndicator

<img width="277" height="82" alt="image" src="https://github.com/user-attachments/assets/0365c7b2-b4fa-4cfc-a9d4-a43d7d584ba5" />

A lightweight Windows system tray application that displays an icon corresponding to your current virtual desktop. It also provides configurable keyboard shortcuts for switching desktops and moving windows between them.

(This readme file is vibed, but I have double-checked that it is correct.)

## Features

- **Tray icon per desktop** — assign a custom icon to each virtual desktop so you always know which one is active.
- **Desktop switching hotkeys** — define keyboard shortcuts to jump to a specific desktop instantly.
- **Window-move hotkeys** — move the currently focused window to another desktop, with an option to follow it automatically.
- **Task View on click** — left-clicking the tray icon opens the Windows Task View.


## Requirements

- Windows 10 / 11 with virtual desktops enabled.
- Rust toolchain targeting `x86_64-pc-windows-gnu` (or `msvc`).

## Building

```sh
cargo build --release
```

## Configuration

The application reads its settings from a YAML file located at:

```
%USERPROFILE%\desktop-indicator.yaml
```

### Example configuration

```yaml
default_icon_path: "C:/icons/default.ico"

desktop_index_to_icon_path:
  0: "C:/icons/desktop1.ico"
  1: "C:/icons/desktop2.ico"
  2: "C:/icons/desktop3.ico"

switch_desktop_hotkeys:
  - modifier_keys: ["Alt"]
    trigger_key: "1"
    target_desktop_index: 0
  - modifier_keys: ["Alt"]
    trigger_key: "2"
    target_desktop_index: 1
  - modifier_keys: ["Alt"]
    trigger_key: "3"
    target_desktop_index: 2

move_window_hotkeys:
  - modifier_keys: ["Alt", "Shift"]
    trigger_key: "1"
    target_desktop_index: 0
  - modifier_keys: ["Alt", "Shift"]
    trigger_key: "2"
    target_desktop_index: 1
  - modifier_keys: ["Alt", "Shift"]
    trigger_key: "3"
    target_desktop_index: 2

follow_moved_windows: true
```

| Field | Description |
|---|---|
| `default_icon_path` | Path to the icon shown when no desktop-specific icon is configured. |
| `desktop_index_to_icon_path` | Map of zero-based desktop index to icon file path. |
| `switch_desktop_hotkeys` | List of hotkeys that switch to a target desktop. |
| `move_window_hotkeys` | List of hotkeys that move the active window to a target desktop. |
| `follow_moved_windows` | If `true`, the view follows the window to the target desktop after moving it. |

## Usage

1. Create the configuration file as described above.
2. Run the application:
   ```sh
   cargo run --release
   ```
   Or launch the compiled binary directly from `target/release/`.
3. The tray icon will appear in the system tray and update automatically when you switch desktops.
4. Right-click the tray icon and select **Exit** to quit.
