# c2-logs

Capture, filter and analyze logs from [Chatterino](https://chatterino.com) without restarting the application on Windows. You can download a prebuilt application from the [releases tab](https://github.com/Nerixyz/c2-logs/releases). This program uses [Windows' Debugger API](https://learn.microsoft.com/en-us/windows/win32/api/debugapi/) to capture logs from Chatterino.

## Usage

1. Open Chatterino regularly
2. Run `c2-logs` (or `./c2-logs.exe`)

You can filter the logs by providing arguments. The default mode is `exclude`, meaning any category you provide will be excluded.

**Example:** `chatterino.http` is usually a bit spammy, and maybe you don't want to see `chatterino.irc` either. To exclude these categories, run `c2-logs http irc`.

You can change the mode to `include`, meaning only logs in the categories you provide are included.

**Example:** To only include `chatterino.twitch` and `chatterino.irc`, run `c2-logs -m include twitch irc`.

If you installed Chatterino with a different executable name, you can pass this using `--exe <name>`.
If you have multiple instances open and want to select a specific one, then you can specify a process-id using `--pid <id>`.

## Building

To build the program, you need to have [Rust](https://www.rust-lang.org/) installed.

```powershell
cargo build -r
```

### Installing

To install the program, run

```powershell
cargo install --path .
```
