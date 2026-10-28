# Moonbridge

A Debug Adapter Protocol (DAP) wrapper for Lua, written in Rust, based on [Local Lua Debugger](https://github.com/tomblind/local-lua-debugger-vscode), which is freely available under the MIT licence.

## Requirements
- Rust
- a lua interpreter of some kind.

## Install with Cargo

```sh
cargo install moonbridge
```

## Build from source

```sh
git clone https://github.com/RubyNova/moonbridge moonbridge
cd moonbridge
cargo build --release
```

The executable is `target/release/moonbridge`. To install it onto your `PATH`:

```sh
cargo install --path . --locked
```

## Usage

Configure your editor to launch Moonbridge as a stdio DAP adapter. For example, if you are using Neovim with `nvim-dap`:

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
    luaExecutable = 'lua', -- Default; this setting can be omitted.
    cwd = '${workspaceFolder}',
    stopOnEntry = false,
  },
}
```

You can then open a Lua file, set a breakpoint, and start the **Debug Lua** configuration through `nvim-dap`.

By default, Moonbridge runs `lua` from `PATH`. The Lua version depends on your installation and the environment your editor runs in;
Moonbridge does not select a particular version or install Lua for you. Set `luaExecutable` to another installed command, such as `lua5.5` or `luajit`, or an absolute interpreter path.
If no `lua` executable is available on `PATH`, you **must** set this option or install Lua.

Optional launch settings include `args` (script arguments) and `env` (environment variables).
