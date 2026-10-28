use crate::{dap_protocol::LaunchRequestArguments, error::Result};
use serde::Deserialize;
use std::process::{Child, Command, Stdio};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArguments {
    #[serde(flatten)]
    pub request: LaunchRequestArguments,
    pub program: Program,
    #[serde(default = "default_lua")]
    pub lua_executable: String,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub stop_on_entry: bool,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub script_roots: Option<Vec<String>>,
    pub break_in_coroutines: Option<bool>,
    pub step_unmapped_lines: Option<bool>,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

// Accept both a simple DAP path and the original adapter's interpreter config.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Program {
    Path(PathBuf),
    Interpreter {
        file: PathBuf,
        lua: Option<String>,
        communication: Option<String>,
    },
}

impl LaunchArguments {
    fn program_path(&self) -> &Path {
        match &self.program {
            Program::Path(path) => path,
            Program::Interpreter { file, .. } => file,
        }
    }

    fn executable(&self) -> &str {
        match &self.program {
            Program::Interpreter { lua: Some(lua), .. } => lua,
            _ => &self.lua_executable,
        }
    }
}

fn default_lua() -> String {
    "lua".into()
}

pub fn spawn(arguments: &LaunchArguments) -> Result<Child> {
    if let Program::Interpreter {
        communication: Some(method),
        ..
    } = &arguments.program
        && method != "stdio"
    {
        return Err(crate::error::MoonbridgeError::InvalidMessage(
            "Only stdio communication is supported".into(),
        ));
    }

    // Lua 5.1 does not populate arg during -e. Pass the path through the
    // environment and build an explicitly escaped argument table instead.
    // Embedding the helper makes installed binaries independent of the source tree.
    let args = arguments
        .args
        .iter()
        .map(|arg| lua_string(arg))
        .collect::<Vec<_>>()
        .join(",");

    let bootstrap = format!(
        "local debugger = (function(...) {} end)(); local argv = {{{args}}}; argv[0] = os.getenv('MOONBRIDGE_PROGRAM'); debugger.runFile(argv[0], true, argv); os.exit(0)",
        include_str!("../lldebugger.lua")
    );

    let mut command = Command::new(arguments.executable());

    command
        .arg("-e")
        .arg(bootstrap)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .envs(&arguments.env)
        .env("MOONBRIDGE_PROGRAM", arguments.program_path())
        .env("LOCAL_LUA_DEBUGGER_VSCODE", "1")
        .env_remove("LOCAL_LUA_DEBUGGER_INPUT_FILE")
        .env_remove("LOCAL_LUA_DEBUGGER_OUTPUT_FILE");

    if let Some(roots) = &arguments.script_roots {
        command.env("LOCAL_LUA_DEBUGGER_SCRIPT_ROOTS", roots.join(";"));
    }

    if let Some(enabled) = arguments.break_in_coroutines {
        command.env(
            "LOCAL_LUA_DEBUGGER_BREAK_IN_COROUTINES",
            if enabled { "1" } else { "0" },
        );
    }

    if let Some(enabled) = arguments.step_unmapped_lines {
        command.env(
            "LOCAL_LUA_DEBUGGER_STEP_UNMAPPED_LINES",
            if enabled { "1" } else { "0" },
        );
    }

    if let Some(cwd) = &arguments.cwd {
        command.current_dir(cwd);
    }

    Ok(command.spawn()?)
}

// Three-digit decimal escapes are supported by every Lua version, unlike JSON
// escapes such as \uXXXX. Encoding bytes also preserves UTF-8 without ambiguity.
fn lua_string(text: &str) -> String {
    let mut literal = String::from("\"");

    for byte in text.bytes() {
        literal.push_str(&format!("\\{byte:03}"));
    }

    literal.push('"');
    literal
}
