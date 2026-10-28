use crate::dap_protocol as dap;
use crate::error::{MoonbridgeError, Result};
use crate::lua_process;
use crate::lua_protocol as lua;
use crate::transport;

use serde_json::{Value, json};

use std::{
    collections::{HashMap, VecDeque},
    io::{BufReader, Read, Write},
    process::{Child, ChildStdin},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

enum Input {
    Request(dap::Request),
    Eof,
    Lua(Value),
    Output(&'static str, Vec<u8>),
    LuaEof,
    Failure(String),
}

struct Session {
    tx: Sender<Input>,
    rx: Receiver<Input>,
    queued: VecDeque<dap::Request>,
    output: std::io::BufWriter<std::io::Stdout>,
    seq: i64,

    thread: i64,

    child: Option<Child>,
    stdin: Option<ChildStdin>,

    stopped: bool,
    configured: bool,
    entry: bool,
    initial_break: bool,
    terminated: bool,
    client_closed: bool,

    breakpoints: HashMap<String, Vec<i64>>,

    references: HashMap<i64, (i64, String)>,
    next_reference: i64,

    frames: HashMap<i64, (i64, usize)>,
    next_frame: i64,

    selected_thread: i64,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stdin.take();

        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn run() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();

    std::thread::spawn(move || {
        let mut reader = BufReader::new(std::io::stdin());

        loop {
            let event = match transport::read_dap_message(&mut reader) {
                Ok(Some(value)) => match serde_json::from_value(value) {
                    Ok(request) => Input::Request(request),
                    Err(error) => Input::Failure(error.to_string()),
                },
                Ok(None) => Input::Eof,
                Err(error) => Input::Failure(error.to_string()),
            };

            let finished = !matches!(event, Input::Request(_));

            if input_tx.send(event).is_err() || finished {
                break;
            }
        }
    });

    let mut session = Session {
        tx,
        rx,
        queued: VecDeque::new(),
        output: std::io::BufWriter::new(std::io::stdout()),
        seq: 1,

        thread: 1,

        child: None,
        stdin: None,

        stopped: false,
        configured: false,
        entry: false,
        initial_break: true,
        terminated: false,
        client_closed: false,

        breakpoints: HashMap::new(),

        references: HashMap::new(),
        next_reference: 1,

        frames: HashMap::new(),
        next_frame: 0,

        selected_thread: 1,
    };

    while !session.client_closed {
        let input = if let Some(request) = session.queued.pop_front() {
            Input::Request(request)
        } else {
            session
                .rx
                .recv()
                .map_err(|e| MoonbridgeError::Channel(e.to_string()))?
        };

        match input {
            Input::Request(request) => {
                let command = request.command.clone();

                match session.request(&command, request.arguments.as_ref().unwrap_or(&Value::Null))
                {
                    Ok(body) => {
                        session.respond(&request, true, body, None)?;

                        if command == "initialize" {
                            session.event("initialized", json!({}))?;
                        }
                    }
                    Err(error) => session.respond(
                        &request,
                        false,
                        Value::Null,
                        Some(dap::ResponseMessage::Other(error.to_string())),
                    )?,
                }

                if command == "disconnect" {
                    break;
                }
            }

            Input::Eof => break,
            other => session.handle(other)?,
        }
    }

    Ok(())
}

fn invalid(message: impl Into<String>) -> MoonbridgeError {
    MoonbridgeError::InvalidMessage(message.into())
}

// Keep internal IDs unchanged; reject out-of-range values at the DAP boundary.
fn dap_integer(value: i64) -> Result<i32> {
    i32::try_from(value).map_err(|_| {
        invalid(format!(
            "Value {value} exceeds the protocol model's i32 range"
        ))
    })
}

fn line_safe(text: &str) -> Result<&str> {
    if text.contains(['\n', '\r']) {
        Err(invalid("Debugger commands cannot contain newlines"))
    } else {
        Ok(text)
    }
}

impl Session {
    fn send(&mut self, mut value: Value) -> Result<()> {
        value["seq"] = json!(self.seq);
        self.seq += 1;

        transport::write_dap_message(&mut self.output, &value)
    }

    fn event(&mut self, event: &str, body: Value) -> Result<()> {
        let event = dap::Event {
            header: dap::MessageHeader {
                seq: 0, // send() assigns the outgoing sequence number.
                message_type: dap::MessageType::Event,
            },
            event: event.to_owned(),
            body: Some(body),
        };

        self.send(serde_json::to_value(event)?)
    }

    fn respond(
        &mut self,
        request: &dap::Request,
        success: bool,
        body: Value,
        message: Option<dap::ResponseMessage>,
    ) -> Result<()> {
        let response = dap::Response {
            header: dap::MessageHeader {
                seq: 0, // send() assigns the outgoing sequence number.
                message_type: dap::MessageType::Response,
            },
            request_seq: request.header.seq,
            success,
            command: request.command.clone(),
            message,
            body: if body.is_null() { None } else { Some(body) },
        };

        self.send(serde_json::to_value(response)?)
    }

    fn command(&mut self, command: &str) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| invalid("No Lua session is running"))?;

        writeln!(stdin, "{}", line_safe(command)?)?;
        stdin.flush()?;

        Ok(())
    }

    fn query(&mut self, command: &str, expected: &str) -> Result<Value> {
        if !self.stopped {
            return Err(invalid("Lua must be stopped for this request"));
        }

        self.command(command)?;

        loop {
            let input = match self.rx.recv_timeout(Duration::from_secs(10)) {
                Ok(input) => input,
                Err(error) => {
                    self.stdin.take();

                    if let Some(child) = &mut self.child {
                        let _ = child.kill();
                    }

                    self.finish()?;

                    return Err(invalid(format!("Waiting for Lua: {error}")));
                }
            };

            match input {
                Input::Request(request) => self.queued.push_back(request),
                Input::Lua(value) if value["type"] == expected => return Ok(value),
                Input::Lua(value) if value["type"] == "error" => {
                    let error: lua::ErrorBody = serde_json::from_value(value)?;

                    return Err(invalid(error.error));
                }

                Input::Eof => {
                    self.client_closed = true;

                    return Err(invalid("Client disconnected"));
                }

                Input::LuaEof => {
                    self.finish()?;

                    return Err(invalid("Lua exited while handling request"));
                }
                other => self.handle(other)?,
            }
        }
    }

    fn handle(&mut self, input: Input) -> Result<()> {
        match input {
            Input::Lua(value) if value["type"] == "debugBreak" => {
                let body: lua::DebugBreakBody = serde_json::from_value(value)?;

                self.stopped = true;
                self.thread = i64::from(body.thread_id);
                self.selected_thread = self.thread;

                if self.initial_break {
                    self.initial_break = false;

                    if !self.configured {
                        return Ok(());
                    }

                    if !self.entry {
                        self.resume("continue")?;

                        return Ok(());
                    }
                }

                let reason = match body.break_type.as_str() {
                    "breakpoint" => "breakpoint",
                    "error" => "exception",
                    "step" => "step",
                    _ => "entry",
                };

                let stopped = dap::StoppedEventBody {
                    reason: reason.into(),
                    thread_id: Some(dap_integer(self.thread)?),
                    all_threads_stopped: Some(true),
                    description: Some(body.message),
                    ..Default::default()
                };

                self.event("stopped", serde_json::to_value(stopped)?)?;
            }

            Input::Output(category, bytes) => {
                let output = dap::OutputEventBody {
                    category: Some(category.into()),
                    output: String::from_utf8_lossy(&bytes).into_owned(),
                    ..Default::default()
                };

                self.event("output", serde_json::to_value(output)?)?;
            }

            Input::LuaEof => self.finish()?,
            Input::Failure(message) => return Err(invalid(message)),
            Input::Lua(value) if value["type"] == "error" => {
                let error: lua::ErrorBody = serde_json::from_value(value)?;

                let output = dap::OutputEventBody {
                    category: Some("stderr".into()),
                    output: error.error,
                    ..Default::default()
                };

                self.event("output", serde_json::to_value(output)?)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.terminated {
            return Ok(());
        }

        self.terminated = true;
        self.stopped = false;
        self.stdin.take();

        let code = if let Some(child) = &mut self.child {
            child.wait()?.code().unwrap_or(1)
        } else {
            0
        };

        let exited = dap::ExitedEventBody { exit_code: code };
        self.event("exited", serde_json::to_value(exited)?)?;

        let terminated = dap::TerminatedEventBody { restart: None };
        self.event("terminated", serde_json::to_value(terminated)?)
    }

    fn resume(&mut self, command: &str) -> Result<()> {
        if !self.stopped {
            return Err(invalid("Lua is already running"));
        }

        self.command(command)?;
        self.stopped = false;
        self.references.clear();
        self.frames.clear();

        let continued = dap::ContinuedEventBody {
            thread_id: dap_integer(self.thread)?,
            all_threads_continued: Some(true),
        };

        self.event("continued", serde_json::to_value(continued)?)
    }

    fn reference(&mut self, frame: i64, expression: String) -> i64 {
        let id = self.next_reference;
        self.next_reference += 1;
        self.references.insert(id, (frame, expression));

        id
    }

    fn select_frame(&mut self, frame: i64) -> Result<()> {
        let (thread, index) = self
            .frames
            .get(&frame)
            .copied()
            .ok_or_else(|| invalid("Unknown or expired frameId"))?;

        self.query(&format!("thread {thread}"), "stack")?;
        self.selected_thread = thread;
        self.query(&format!("frame {}", index + 1), "stack")?;

        Ok(())
    }

    fn request(&mut self, command: &str, args: &Value) -> Result<Value> {
        match command {
            "initialize" => {
                let arguments: dap::InitializeRequestArguments =
                    serde_json::from_value(args.clone())?;

                if matches!(arguments.path_format, Some(dap::PathFormat::Uri))
                    || arguments.lines_start_at_1 == Some(false)
                    || arguments.columns_start_at_1 == Some(false)
                {
                    return Err(invalid(
                        "Only filesystem paths and one-based coordinates are supported",
                    ));
                }

                let capabilities = dap::Capabilities {
                    supports_configuration_done_request: Some(true),
                    supports_conditional_breakpoints: Some(true),
                    supports_terminate_request: Some(true),

                    ..Default::default()
                };

                Ok(serde_json::to_value(capabilities)?)
            }
            "launch" => {
                let arguments: lua_process::LaunchArguments = serde_json::from_value(args.clone())?;

                if arguments.request.no_debug == Some(true) {
                    return Err(invalid("noDebug launches are not supported"));
                }

                if self.child.is_some() {
                    return Err(invalid("A session has already been launched"));
                }

                for pattern in &arguments.ignore_patterns {
                    line_safe(pattern)?;
                }

                self.entry = arguments.stop_on_entry;

                let mut child = lua_process::spawn(&arguments)?;
                self.stdin = child.stdin.take();
                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();

                let tx = self.tx.clone();

                std::thread::spawn(move || {
                    let result = transport::read_lua_stream(stdout, |protocol, bytes| {
                        let input = if protocol {
                            match serde_json::from_slice(&bytes) {
                                Ok(value) => Input::Lua(value),
                                Err(e) => Input::Failure(format!("Invalid Lua JSON: {e}")),
                            }
                        } else {
                            Input::Output("stdout", bytes)
                        };

                        let _ = tx.send(input);
                    });

                    if let Err(e) = result {
                        let _ = tx.send(Input::Failure(e.to_string()));
                    }

                    let _ = tx.send(Input::LuaEof);
                });

                let tx = self.tx.clone();

                std::thread::spawn(move || {
                    let mut stderr = stderr;
                    let mut bytes = [0; 4096];

                    while let Ok(n) = stderr.read(&mut bytes) {
                        if n == 0 {
                            break;
                        }

                        if tx
                            .send(Input::Output("stderr", bytes[..n].to_vec()))
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                self.child = Some(child);

                // Wait for the helper's initial stop before allowing configuration.
                while self.initial_break {
                    let input = self
                        .rx
                        .recv_timeout(Duration::from_secs(10))
                        .map_err(|e| invalid(format!("Starting Lua: {e}")))?;

                    match input {
                        Input::Request(request) => self.queued.push_back(request),
                        Input::Eof => {
                            self.client_closed = true;

                            return Err(invalid("Client disconnected"));
                        }
                        other => self.handle(other)?,
                    }

                    if self.terminated {
                        return Err(invalid("Lua exited before reaching the program"));
                    }
                }

                for pattern in &arguments.ignore_patterns {
                    self.query(&format!("ignore {pattern}"), "result")?;
                }

                Ok(json!({}))
            }
            "configurationDone" => {
                if !self.stopped {
                    return Err(invalid("Launch must finish before configurationDone"));
                }

                self.configured = true;

                if self.entry {
                    let stopped = dap::StoppedEventBody {
                        reason: "entry".into(),
                        thread_id: Some(dap_integer(self.thread)?),
                        all_threads_stopped: Some(true),
                        ..Default::default()
                    };

                    self.event("stopped", serde_json::to_value(stopped)?)?;
                } else {
                    self.resume("continue")?;
                }

                Ok(json!({}))
            }
            "setBreakpoints" => {
                let arguments: dap::SetBreakpointsArguments = serde_json::from_value(args.clone())?;
                let path = line_safe(
                    arguments
                        .source
                        .path
                        .as_deref()
                        .ok_or_else(|| invalid("A source path is required"))?,
                )?
                .to_owned();
                let requested = arguments.breakpoints.unwrap_or_default();

                for breakpoint in &requested {
                    if breakpoint.line <= 0 {
                        return Err(invalid("Breakpoint lines must be positive integers"));
                    }

                    if breakpoint.hit_condition.is_some() || breakpoint.log_message.is_some() {
                        return Err(invalid("Hit conditions and logpoints are not supported"));
                    }

                    if let Some(condition) = &breakpoint.condition {
                        line_safe(condition)?;
                    }
                }

                let old = self.breakpoints.remove(&path).unwrap_or_default();

                for line in old {
                    self.query(&format!("break del {path}:{line}"), "result")?;
                }

                let mut lines = Vec::new();
                let mut result = Vec::new();

                for bp in requested {
                    let line = bp.line;
                    let condition = bp.condition.as_deref().unwrap_or("");

                    match self.query(
                        &format!("break set {path}:{line} {condition}"),
                        "breakpoints",
                    ) {
                        Ok(_) => {
                            lines.push(i64::from(line));
                            result.push(dap::Breakpoint {
                                verified: true,
                                line: Some(line),
                                ..Default::default()
                            });
                        }
                        Err(error) => result.push(dap::Breakpoint {
                            verified: false,
                            line: Some(line),
                            message: Some(error.to_string()),
                            ..Default::default()
                        }),
                    }
                }

                self.breakpoints.insert(path, lines);

                let body = dap::SetBreakpointsResponseBody {
                    breakpoints: result,
                };

                Ok(serde_json::to_value(body)?)
            }
            "setExceptionBreakpoints" => {
                let arguments: dap::SetExceptionBreakpointsArguments =
                    serde_json::from_value(args.clone())?;

                if !arguments.filters.is_empty() {
                    return Err(invalid("Exception filters are not supported"));
                }

                let body = dap::SetExceptionBreakpointsResponseBody {
                    breakpoints: Vec::new(),
                };

                Ok(serde_json::to_value(body)?)
            }
            "threads" => {
                let result = self.query("threads", "threads")?;
                let result: lua::ThreadsBody = serde_json::from_value(result)?;

                let threads = result
                    .threads
                    .into_iter()
                    .map(|thread| dap::Thread {
                        id: thread.id,
                        name: thread.name,
                    })
                    .collect();

                let body = dap::ThreadsResponseBody { threads };

                Ok(serde_json::to_value(body)?)
            }
            "stackTrace" => {
                let arguments: dap::StackTraceArguments = serde_json::from_value(args.clone())?;
                let thread = i64::from(arguments.thread_id);
                self.query(&format!("thread {thread}"), "stack")?;
                self.selected_thread = thread;

                let result = self.query("stack", "stack")?;
                let result: lua::StackBody = serde_json::from_value(result)?;
                let frames = result.frames;

                let start = arguments
                    .start_frame
                    .and_then(|n| usize::try_from(n).ok())
                    .unwrap_or(0);
                let count = arguments
                    .levels
                    .filter(|n| *n > 0)
                    .map(|n| n as usize)
                    .unwrap_or(frames.len());

                let mut stack_frames = Vec::new();

                for (index, frame) in frames.iter().enumerate().skip(start).take(count) {
                    let id = self.next_frame;
                    self.next_frame += 1;
                    self.frames.insert(id, (self.selected_thread, index));

                    let source = dap::Source {
                        path: Some(frame.source.clone()),
                        ..Default::default()
                    };

                    stack_frames.push(dap::StackFrame {
                        id: dap_integer(id)?,
                        name: frame.func.clone().unwrap_or_else(|| "<chunk>".into()),
                        source: Some(source),
                        line: frame.line.max(1),
                        column: 1,
                        ..Default::default()
                    });
                }

                let body = dap::StackTraceResponseBody {
                    stack_frames,
                    total_frames: Some(dap_integer(frames.len() as i64)?),
                };

                Ok(serde_json::to_value(body)?)
            }
            "scopes" => {
                if !self.stopped {
                    return Err(invalid("Lua must be stopped"));
                }

                let arguments: dap::ScopesArguments = serde_json::from_value(args.clone())?;

                let frame = i64::from(arguments.frame_id);

                if !self.frames.contains_key(&frame) {
                    return Err(invalid("Unknown or expired frameId"));
                }

                let locals = self.reference(frame, "@locals".into());
                let ups = self.reference(frame, "@ups".into());
                let globals = self.reference(frame, "@globals".into());

                let body = dap::ScopesResponseBody {
                    scopes: vec![
                        dap::Scope {
                            name: "Locals".into(),
                            presentation_hint: Some(dap::ScopePresentationHint::Locals),
                            variables_reference: dap_integer(locals)?,
                            expensive: false,
                            ..Default::default()
                        },
                        dap::Scope {
                            name: "Upvalues".into(),
                            variables_reference: dap_integer(ups)?,
                            expensive: false,
                            ..Default::default()
                        },
                        dap::Scope {
                            name: "Globals".into(),
                            variables_reference: dap_integer(globals)?,
                            expensive: true,
                            ..Default::default()
                        },
                    ],
                };

                Ok(serde_json::to_value(body)?)
            }
            "variables" => {
                let arguments: dap::VariablesArguments = serde_json::from_value(args.clone())?;
                let id = i64::from(arguments.variables_reference);

                let (frame, expression) = self
                    .references
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| invalid("Unknown or expired variablesReference"))?;

                self.select_frame(frame)?;

                let scope = expression.starts_with('@');

                let result = if scope {
                    self.query(&expression[1..], "variables")?
                } else {
                    self.query(&format!("props {expression} all"), "properties")?
                };

                let values = if scope {
                    let body: lua::VariablesBody = serde_json::from_value(result)?;
                    body.variables
                } else {
                    let body: lua::PropertiesBody = serde_json::from_value(result)?;
                    body.properties
                };

                let mut variables = Vec::new();

                for value in values {
                    let expr = if scope {
                        value.name.clone()
                    } else {
                        format!("({expression})[{}]", value.name)
                    };

                    let reference = if value.var_type == "table" {
                        self.reference(frame, expr.clone())
                    } else {
                        0
                    };

                    variables.push(dap::Variable {
                        name: value.name,
                        value: value.value,
                        variable_data_type: Some(value.var_type),
                        variables_reference: Some(dap_integer(reference)?),
                        evaluate_name: Some(expr),
                        ..Default::default()
                    });
                }

                let body = dap::VariablesResponseBody { variables };

                Ok(serde_json::to_value(body)?)
            }
            "evaluate" => {
                let arguments: dap::EvaluateArguments = serde_json::from_value(args.clone())?;
                let expression = line_safe(&arguments.expression)?;

                let frame = if let Some(frame) = arguments.frame_id {
                    i64::from(frame)
                } else {
                    let id = self.next_frame;
                    self.next_frame += 1;
                    self.frames.insert(id, (self.thread, 0));
                    id
                };

                self.select_frame(frame)?;

                let result = self.query(&format!("eval {expression}"), "result")?;
                let result: lua::ResultBody = serde_json::from_value(result)?;
                let value = result.results.into_iter().next();

                let reference = if value
                    .as_ref()
                    .is_some_and(|value| value.value_type == "table")
                {
                    self.reference(frame, expression.into())
                } else {
                    0
                };

                let body = dap::EvaluateResponseBody {
                    result: value
                        .as_ref()
                        .map(|value| value.value.clone())
                        .unwrap_or_else(|| "nil".into()),
                    result_type: value.map(|value| value.value_type),
                    variables_reference: dap_integer(reference)?,
                };

                Ok(serde_json::to_value(body)?)
            }
            "continue" | "next" | "stepIn" | "stepOut" => {
                let lua = match command {
                    "next" => "step",
                    "stepIn" => "stepin",
                    "stepOut" => "stepout",
                    _ => "continue",
                };

                self.resume(lua)?;

                let body = dap::ContinueResponseBody {
                    all_threads_continued: Some(true),
                };

                Ok(serde_json::to_value(body)?)
            }
            "disconnect" | "terminate" => {
                self.stdin.take();

                if let Some(child) = &mut self.child {
                    child.kill()?;
                }

                self.finish()?;

                Ok(json!({}))
            }
            _ => Err(invalid(format!("Unsupported request: {command}"))),
        }
    }
}
