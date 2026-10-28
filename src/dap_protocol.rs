use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Request,
    Response,
    Event,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseMessageType {
    Cancelled,
    NotStopped,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseMessage {
    Known(ResponseMessageType),
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PathFormat {
    Path,
    Uri,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourcePresentationHint {
    Normal,
    Emphasize,
    Deemphasize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StackFramePresentationHint {
    Normal,
    Label,
    Subtle,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScopePresentationHint {
    Arguments,
    Locals,
    Registers,
    ReturnValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    MD5,
    SHA1,
    SHA256,
    #[serde(rename = "timestamp")]
    Timestamp,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SteppingGranularity {
    Statement,
    Line,
    Instruction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VariableFilter {
    Indexed,
    Named,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColumnDescriptorDataType {
    String,
    Number,
    Boolean,
    #[serde(rename = "unixTimestampUTC")]
    UnixTimestampUTC,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakpointModeApplicability {
    Source,
    Exception,
    Data,
    Instruction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakpointUnverifiedReason {
    Pending,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModuleId {
    Number(i32),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageHeader {
    pub seq: i32,

    #[serde(rename = "type")]
    pub message_type: MessageType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(flatten)]
    pub header: MessageHeader,

    pub command: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event<T> {
    #[serde(flatten)]
    pub header: MessageHeader,

    pub event: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    #[serde(flatten)]
    pub header: MessageHeader,

    #[serde(rename = "request_seq")]
    pub request_seq: i32,
    pub success: bool,
    pub command: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<ResponseMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseMessageBody {
    pub id: i32,
    pub format: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<std::collections::HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_telemetry: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_user: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_label: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<SourcePresentationHint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<Source>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<Checksum>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    pub line: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArguments {
    pub source: Source,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_modified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestArguments {
    #[serde(rename = "clientID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    #[serde(rename = "adapterID")]
    // The adapter has historically accepted initialize requests without an ID.
    #[serde(default)]
    pub adapter_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_start_at_1: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_start_at_1: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_format: Option<PathFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_type: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_paging: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_run_in_terminal_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_references: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_progress_reporting: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_invalidated_event: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_event: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_args_can_be_interpreted_by_shell: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_start_debugging_request: Option<bool>,

    #[serde(rename = "supportsANSIStyling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_ansi_styling: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_debug: Option<bool>,

    #[serde(rename = "__restart")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadHeaderArguments {
    pub thread_id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueArguments {
    #[serde(flatten)]
    pub thread_header_arguments: ThreadHeaderArguments,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextArguments {
    #[serde(flatten)]
    pub thread_header_arguments: ThreadHeaderArguments,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInArguments {
    #[serde(flatten)]
    pub thread_header_arguments: ThreadHeaderArguments,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepOutArguments {
    #[serde(flatten)]
    pub thread_header_arguments: ThreadHeaderArguments,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceArguments {
    pub thread_id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StackFrameFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueFormat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameFormat {
    #[serde(flatten)]
    pub value_format_members: ValueFormat,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_types: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_names: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_all: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesArguments {
    pub frame_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesArguments {
    pub variables_reference: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<VariableFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisconnectArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_debuggee: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_debuggee: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminateArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionBreakpointsFilter {
    pub filter: String,
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_condition: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDescriptor {
    pub attribute_name: String,
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_data_type: Option<ColumnDescriptorDataType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointMode {
    pub mode: String,
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub applies_to: Vec<BreakpointModeApplicability>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_configuration_done_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_function_breakpoints: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_conditional_breakpoints: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_hit_conditional_breakpoints: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_evaluate_for_hovers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_breakpoint_filters: Option<Vec<ExceptionBreakpointsFilter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_back: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_set_variable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_restart_frame: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_goto_targets_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_in_targets_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_completions_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_trigger_characters: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_modules_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_module_columns: Option<Vec<ColumnDescriptor>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_checksum_algorithms: Option<Vec<ChecksumAlgorithm>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_restart_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_exception_options: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_value_formatting_options: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_exception_info_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_terminate_debuggee: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_suspend_debugee: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_delayed_stack_trace_loading: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_loaded_sources_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_log_points: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_terminate_threads_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_set_expression: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_terminate_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_data_breakpoints: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_read_memory_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_write_memory_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_disassemble_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_cancel_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_breakpoint_locations_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_clipboard_context: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_stepping_granularity: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_instruction_breakpoints: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_exception_filter_options: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_single_thread_execution_requests: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_data_breakpoint_bytes: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoint_modes: Option<Vec<BreakpointMode>>,

    #[serde(rename = "supportsANSIStyling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_ansi_styling: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    pub verified: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reference: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<BreakpointUnverifiedReason>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsResponseBody {
    pub breakpoints: Vec<Breakpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_continued: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub id: i32,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    pub line: i32,
    pub column: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_restart: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_pointer_reference: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_id: Option<ModuleId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<StackFramePresentationHint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceResponseBody {
    pub stack_frames: Vec<StackFrame>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_frames: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<ScopePresentationHint>,

    pub variables_reference: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i32>,

    pub expensive: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesResponseBody {
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablePresentationHint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub variable_data_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<VariablePresentationHint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reference: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_location_reference: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_location_reference: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesResponseBody {
    pub variables: Vec<Variable>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsResponseBody {
    pub threads: Vec<Thread>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoppedEventBody {
    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_focus_hint: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_stopped: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_breakpoint_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuedEventBody {
    pub thread_id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_continued: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitedEventBody {
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatedEventBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEventBody {
    pub output: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// These supported messages were not included in the original declarations.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateArguments {
    pub expression: String,
    pub frame_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponseBody {
    pub result: String,

    #[serde(rename = "type")]
    pub result_type: Option<String>,

    pub variables_reference: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExceptionBreakpointsArguments {
    #[serde(default)]
    pub filters: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExceptionBreakpointsResponseBody {
    pub breakpoints: Vec<Breakpoint>,
}

#[cfg(test)]
mod tests {
    use super::{ResponseMessage, ResponseMessageType};
    use serde_json::json;

    // Known error names and other error text must both be sent as plain strings.
    #[test]
    fn response_messages_round_trip_as_plain_strings() {
        for (message, expected) in [
            (
                ResponseMessage::Known(ResponseMessageType::Cancelled),
                "cancelled",
            ),
            (
                ResponseMessage::Known(ResponseMessageType::NotStopped),
                "notStopped",
            ),
            (
                ResponseMessage::Other("Unknown or expired frameId".into()),
                "Unknown or expired frameId",
            ),
        ] {
            let encoded = serde_json::to_value(message).unwrap();

            assert_eq!(encoded, json!(expected));

            let decoded: ResponseMessage = serde_json::from_value(encoded).unwrap();

            // Reading the string back should recover the correct enum variant.
            match (expected, decoded) {
                ("cancelled", ResponseMessage::Known(ResponseMessageType::Cancelled)) => {}
                ("notStopped", ResponseMessage::Known(ResponseMessageType::NotStopped)) => {}
                (text, ResponseMessage::Other(message)) => assert_eq!(message, text),
                (_, unexpected) => panic!("Incorrect message variant: {unexpected:?}"),
            }
        }
    }

    // Numbers, objects and other non-string values are not valid error messages.
    #[test]
    fn response_messages_reject_non_strings() {
        for value in [
            json!(null),
            json!(42),
            json!(false),
            json!([]),
            json!({"Other":"error"}),
        ] {
            assert!(serde_json::from_value::<ResponseMessage>(value).is_err());
        }
    }
}
