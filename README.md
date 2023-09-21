# c2-logs

Capture, filter and analyze logs from [Chatterino](https://chatterino.com) without restarting the application on Windows. You can download a prebuilt application from the [releases tab](https://github.com/Nerixyz/c2-logs/releases). This program uses [Windows' Debugger API](https://learn.microsoft.com/en-us/windows/win32/api/debugapi/) to capture logs from Chatterino and calls [`QLoggingCategory::setFilterRules`](https://doc.qt.io/qt-6/qloggingcategory.html#setFilterRules).

## Usage

1. Open Chatterino regularly
2. Run `c2-logs chatterino.*.debug=true` (if you don't have it in your `PATH`, run it using `.\c2-logs.exe`)

You can specify multiple rules.
To enable debug logging from Chatterino while excluding the `chatterino.http` category, run `c2-logs chatterino.*.debug=true chatterino.http.debug=false`.
Check [Qt's documentation](https://doc.qt.io/qt-6/qloggingcategory.html#configuring-categories) on the logging rules.

```text
Usage: c2-logs.exe [OPTIONS] [RULES]...

Arguments:
  [RULES]...  Qt filter rules (e.g. *.debug=true or foo.bar.debug=false) multiple rules will be joined by a newline

Options:
      --exe <EXECUTABLE>  Use this to specify the name of the chatterino executable. [default: chatterino.exe]
      --pid <PID>         Use this to specify a specific process-id to attach to.
```

## Building

To build the program, you need to have [Rust](https://www.rust-lang.org/) installed.

```powershell
cargo build -r
```

### Installing

To install the program with `cargo`, run

```powershell
cargo install c2-logs
```
