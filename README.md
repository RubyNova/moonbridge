# Moonbridge

A Rust Debug Adapter Protocol (DAP) adapter for Lua, based on
[Local Lua Debugger](https://github.com/tomblind/local-lua-debugger-vscode).

Requires Rust to build or install, and a Lua interpreter to debug your scripts.
The Lua debugger helper is embedded in the executable.

## Install with Cargo

Replace `<repository-url>` with this repository's clone URL:

```sh
cargo install --git <repository-url> --locked
```

This installs `moonbridge` into Cargo's bin directory, normally `~/.cargo/bin`.
Make sure that directory is on your `PATH`.

## Build from source

```sh
git clone <repository-url> moonbridge
cd moonbridge
cargo build --release
```

The executable is `target/release/moonbridge`. To install it onto your `PATH`:

```sh
cargo install --path . --locked
```

## Usage

Configure your editor to launch Moonbridge as a stdio DAP adapter.
For Neovim with `nvim-dap`:

```lua
local dap = require('dap')

dap.adapters.moonbridge = {
  type = 'executable',
  command = 'moonbridge', -- Or the absolute path to target/release/moonbridge.
}

dap.configurations.lua = {
  {
    name = 'Debug Lua',
    type = 'moonbridge',
    request = 'launch',
    program = '${file}',
    luaExecutable = 'lua',
    cwd = '${workspaceFolder}',
    stopOnEntry = false,
  },
}
```

Open a Lua file, set a breakpoint, and start **Debug Lua** through `nvim-dap`.
Set `luaExecutable` to use another interpreter, such as `lua5.5` or `luajit`.
Optional launch settings include `args` (script arguments) and `env`
(environment variables).
