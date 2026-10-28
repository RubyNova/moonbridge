use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LuaMessage {
    #[serde(rename = "debugBreak")]
    DebugBreak(DebugBreakBody),
    #[serde(rename = "error")]
    Error(ErrorBody),
    #[serde(rename = "result")]
    Result(ResultBody),
    #[serde(rename = "stack")]
    Stack(StackBody),
    #[serde(rename = "threads")]
    Threads(ThreadsBody),
    #[serde(rename = "variables")]
    Variables(VariablesBody),
    #[serde(rename = "properties")]
    Properties(PropertiesBody),
    #[serde(rename = "breakpoints")]
    Breakpoints(BreakpointsBody),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugBreakBody {
    pub tag: String,
    pub message: String,
    pub break_type: String,
    pub thread_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub tag: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultBody {
    pub tag: String,
    pub results: Vec<ResultValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappedLocation {
    pub source: String,
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuaStackFrame {
    pub source: String,
    // Placeholder frames from the helper may have no line information.
    #[serde(default)]
    pub line: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub func: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapped_location: Option<MappedLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackBody {
    pub tag: String,
    pub frames: Vec<LuaStackFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuaThread {
    pub name: String,
    pub id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsBody {
    pub tag: String,
    pub threads: Vec<LuaThread>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuaVariable {
    #[serde(rename = "type")]
    pub var_type: String,
    pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesBody {
    pub tag: String,
    pub variables: Vec<LuaVariable>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LengthInfo {
    #[serde(rename = "type")]
    pub value_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetatableInfo {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertiesBody {
    pub tag: String,
    pub properties: Vec<LuaVariable>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metatable: Option<MetatableInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<LengthInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuaBreakpoint {
    pub line: i32,
    pub file: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointsBody {
    pub tag: String,
    pub breakpoints: Vec<LuaBreakpoint>,
}
