use serde_json::{Value, json};

use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

// This helper acts like an editor: it starts Moonbridge and exchanges messages with it.
struct Client {
    child: Child,
    messages: mpsc::Receiver<Value>,
    seq: i64,
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Client {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_moonbridge"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let (tx, messages) = mpsc::channel();

        std::thread::spawn(move || {
            loop {
                let mut header = String::new();

                if stdout.read_line(&mut header).unwrap() == 0 {
                    break;
                }

                // The header tells us how many bytes belong to this JSON message.
                let length: usize = header
                    .trim()
                    .strip_prefix("Content-Length: ")
                    .unwrap()
                    .parse()
                    .unwrap();

                // DAP puts an empty line between the header and the JSON.
                let mut blank = String::new();
                stdout.read_line(&mut blank).unwrap();

                let mut bytes = vec![0; length];
                stdout.read_exact(&mut bytes).unwrap();

                if tx.send(serde_json::from_slice(&bytes).unwrap()).is_err() {
                    break;
                }
            }
        });

        Self {
            child,
            messages,
            seq: 0,
        }
    }

    // Wait for the next message, but fail the test if Moonbridge never replies.
    fn receive(&self) -> Value {
        self.messages
            .recv_timeout(Duration::from_secs(15))
            .expect("adapter did not respond")
    }

    fn send(&mut self, command: &str, arguments: Value) -> i64 {
        self.seq += 1;

        let body = serde_json::to_vec(&json!({
            "seq": self.seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        }))
        .unwrap();

        let stdin = self.child.stdin.as_mut().unwrap();
        write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        stdin.write_all(&body).unwrap();
        stdin.flush().unwrap();

        self.seq
    }

    // Skip other messages until we receive the answer to this request number.
    fn response(&self, seq: i64) -> Value {
        loop {
            let message = self.receive();

            if message["type"] == "response" && message["request_seq"] == seq {
                return message;
            }
        }
    }

    // Send a request, check that it succeeded, and return its result.
    fn request(&mut self, command: &str, arguments: Value) -> Value {
        let seq = self.send(command, arguments);
        let response = self.response(seq);

        assert_eq!(response["success"], true, "{response}");

        response["body"].clone()
    }

    // Skip other messages until Moonbridge announces the event we want.
    fn event(&self, name: &str) -> Value {
        loop {
            let message = self.receive();

            if message["event"] == name {
                return message["body"].clone();
            }
        }
    }
}

#[test]
fn real_lua_debug_session() {
    // Choose a Lua version with LUA_EXECUTABLE, or use the normal lua command.
    // For example: LUA_EXECUTABLE=lua5.4 cargo test --test dap_session
    let lua = std::env::var("LUA_EXECUTABLE").unwrap_or_else(|_| "lua".into());

    assert!(
        Command::new(&lua).arg("-v").output().is_ok(),
        "Install Lua or set LUA_EXECUTABLE before running integration tests"
    );

    let directory = std::env::temp_dir().join(format!("moonbridge-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();

    // Include spaces and quotes in the filename to check that launching still works.
    let script = directory.join("a 'quoted' script.lua");
    std::fs::write(
        &script,
        "local answer = 41\nlocal data = {nested = {value = 9}}\nanswer = answer + 1\nprint('answer', answer, arg[1])\n",
    )
    .unwrap();

    // Start Moonbridge and introduce our pretend editor.
    let mut client = Client::new();
    client.request("initialize", json!({"adapterID":"moonbridge"}));
    client.event("initialized");

    client.request(
        "launch",
        json!({"program":script,"luaExecutable":lua,"args":["hello"]}),
    );

    client.request(
        "setBreakpoints",
        json!({"source":{"path":script},"breakpoints":[{"line":3}]}),
    );

    client.request("configurationDone", json!({}));

    let stop = client.event("stopped");

    assert_eq!(stop["reason"], "breakpoint");

    let thread = &stop["threadId"];
    let threads = client.request("threads", json!({}));

    assert!(!threads["threads"].as_array().unwrap().is_empty());

    let stack = client.request("stackTrace", json!({"threadId":thread}));

    assert_eq!(stack["stackFrames"][0]["line"], 3);

    let scopes = client.request("scopes", json!({"frameId":0}));

    assert_eq!(
        scopes,
        json!({
            "scopes": [
                {
                    "name": "Locals",
                    "presentationHint": "locals",
                    "variablesReference": 1,
                    "expensive": false,
                },
                {
                    "name": "Upvalues",
                    "variablesReference": 2,
                    "expensive": false,
                },
                {
                    "name": "Globals",
                    "variablesReference": 3,
                    "expensive": true,
                },
            ],
        })
    );

    assert_eq!(stack["stackFrames"][0]["column"], 1);
    assert_eq!(stack["stackFrames"][0]["source"]["path"], json!(script));

    // Ask for the stack again, leaving out the first frame.
    // levels=0 means return all remaining frames. The total count should stay the same.
    let paged = client.request(
        "stackTrace",
        json!({"threadId":thread,"startFrame":1,"levels":0}),
    );

    assert_eq!(paged["totalFrames"], stack["totalFrames"]);
    assert_eq!(
        paged["stackFrames"].as_array().unwrap().len(),
        stack["stackFrames"].as_array().unwrap().len() - 1
    );

    let id = &scopes["scopes"][0]["variablesReference"];

    // Send two requests before waiting for either answer.
    // Check that Moonbridge answers each request with the correct request number.
    let first = client.send("variables", json!({"variablesReference":id}));
    let second = client.send("variables", json!({"variablesReference":id}));

    let variables = client.response(first);

    assert_eq!(variables["success"], true);
    assert_eq!(client.response(second)["success"], true);

    // The local answer variable should still be 41 at this breakpoint.
    let values = variables["body"]["variables"].as_array().unwrap();
    assert!(
        values
            .iter()
            .any(|v| v["name"] == "answer" && v["value"] == "41")
    );

    // Expand the data table, then expand the table stored in its nested field.
    let table = values.iter().find(|v| v["name"] == "data").unwrap();
    let properties = client.request(
        "variables",
        json!({"variablesReference":table["variablesReference"]}),
    );

    let nested = &properties["variables"][0];
    assert_eq!(nested["name"], "\"nested\"");

    let inner = client.request(
        "variables",
        json!({"variablesReference":nested["variablesReference"]}),
    );

    assert_eq!(inner["variables"][0]["value"], "9");

    // Evaluating an expression should return its value without changing answer.
    assert_eq!(
        client.request("evaluate", json!({"frameId":0,"expression":"answer + 1"}))["result"],
        "42"
    );

    // Run the next line, which changes answer from 41 to 42.
    client.request("next", json!({"threadId":thread}));
    client.event("stopped");

    // Check that the line we just ran changed answer to 42.
    assert_eq!(
        client.request("evaluate", json!({"expression":"answer"}))["result"],
        "42"
    );

    // Remove the breakpoint and let the program finish.
    client.request(
        "setBreakpoints",
        json!({"source":{"path":script},"breakpoints":[]}),
    );
    client.request("continue", json!({"threadId":thread}));

    // Collect printed output until Moonbridge reports that the session has ended.
    let mut output = String::new();

    loop {
        let message = client.receive();

        if message["event"] == "output" {
            output.push_str(message["body"]["output"].as_str().unwrap());
        }

        if message["event"] == "terminated" {
            break;
        }
    }

    assert!(output.contains("answer\t42\thello"), "{output:?}");

    // An unknown command should get an error response instead of hanging.
    let seq = client.send("notACommand", json!({}));
    assert_eq!(client.response(seq)["success"], false);

    client.request("disconnect", json!({}));

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn entry_errors_termination_and_client_eof() {
    let lua = std::env::var("LUA_EXECUTABLE").unwrap_or_else(|_| "lua".into());

    // This program keeps running so we can check that termination stops it.
    let script = std::env::temp_dir().join(format!("moonbridge-entry-{}.lua", std::process::id()));
    std::fs::write(
        &script,
        "local value = 1\nwhile true do value = value + 1 end\n",
    )
    .unwrap();

    // Start Moonbridge and introduce our pretend editor.
    let mut client = Client::new();
    client.request("initialize", json!({}));
    client.event("initialized");

    client.request(
        "launch",
        json!({
            "program": {
                "file": script.file_name().unwrap().to_str().unwrap(),
                "lua": lua,
            },
            "cwd": script.parent(),
            "env": {"MOONBRIDGE_TEST_VALUE": "configured"},
            "ignorePatterns": ["^/not-this-project/"],
            "stopOnEntry": true,
        }),
    );

    // stopOnEntry should pause the program before it starts running normally.
    let seq = client.send("configurationDone", json!({}));
    assert_eq!(client.event("stopped")["reason"], "entry");
    assert_eq!(client.response(seq)["success"], true);

    // The program should receive the environment variable from our launch settings.
    assert_eq!(
        client.request(
            "evaluate",
            json!({"expression":"os.getenv('MOONBRIDGE_TEST_VALUE')"})
        )["result"],
        "\"configured\""
    );

    // Invalid Lua should produce an error response while keeping the session usable.
    let seq = client.send("evaluate", json!({"expression":"this is invalid lua"}));
    assert_eq!(client.response(seq)["success"], false);

    // A breakpoint request without a source path should also produce an error.
    let seq = client.send("setBreakpoints", json!({"source":{},"breakpoints":[]}));
    assert_eq!(client.response(seq)["success"], false);

    // Send fields with the wrong types and check that each request gets an error.
    for (command, arguments) in [
        ("scopes", json!({"frameId":"wrong type"})),
        ("variables", json!({"variablesReference":null})),
        ("stackTrace", json!({"threadId":"wrong type"})),
        (
            "setBreakpoints",
            json!({
                "source": {"path": script},
                "breakpoints": [{"line": "wrong type"}],
            }),
        ),
    ] {
        let seq = client.send(command, arguments);
        let response = client.response(seq);
        assert_eq!(response["success"], false, "{response}");
        assert!(response["message"].is_string(), "{response}");
    }

    // Let the program run, then ask Moonbridge to stop it.
    client.request("continue", json!({"threadId":2}));
    client.request("terminate", json!({}));

    client.request("disconnect", json!({}));

    // Start another session, then close the editor end of its input pipe.
    let mut eof_client = Client::new();
    eof_client.request("launch", json!({"program":script,"luaExecutable":lua}));

    eof_client.child.stdin.take();

    // Moonbridge should exit on its own rather than waiting forever for more input.
    let deadline = std::time::Instant::now() + Duration::from_secs(3);

    loop {
        if let Some(status) = eof_client.child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }

        assert!(
            std::time::Instant::now() < deadline,
            "adapter hung after client EOF"
        );

        std::thread::sleep(Duration::from_millis(10));
    }

    std::fs::remove_file(script).unwrap();
}
