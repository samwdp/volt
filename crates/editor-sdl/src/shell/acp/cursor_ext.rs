use agent_client_protocol::{Error, ExtResponse, PlanEntry, PlanEntryPriority, PlanEntryStatus};
use serde_json::{Map, Value, value::RawValue};

pub(crate) const CURSOR_ASK_QUESTION: &str = "cursor/ask_question";
pub(crate) const CURSOR_CREATE_PLAN: &str = "cursor/create_plan";
pub(crate) const CURSOR_UPDATE_TODOS: &str = "cursor/update_todos";
pub(crate) const CURSOR_TASK: &str = "cursor/task";
pub(crate) const CURSOR_GENERATE_IMAGE: &str = "cursor/generate_image";

#[derive(Debug, Clone)]
pub(crate) struct CursorQuestion {
    pub(crate) id: String,
    pub(crate) prompt: String,
    pub(crate) options: Vec<CursorOption>,
    pub(crate) allow_multiple: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorOption {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorTodo {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorAsk {
    #[allow(dead_code)]
    pub(crate) tool_call_id: String,
    pub(crate) title: Option<String>,
    pub(crate) questions: Vec<CursorQuestion>,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorPlanRequest {
    pub(crate) name: Option<String>,
    pub(crate) overview: Option<String>,
    pub(crate) plan: String,
    pub(crate) todos: Vec<CursorTodo>,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorTodoUpdate {
    pub(crate) todos: Vec<CursorTodo>,
    pub(crate) merge: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorTaskNotice {
    pub(crate) tool_call_id: String,
    pub(crate) description: String,
    pub(crate) prompt: String,
    pub(crate) subagent_type: String,
    pub(crate) agent_id: Option<String>,
    pub(crate) duration_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorImageNotice {
    pub(crate) description: String,
    pub(crate) file_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct CursorAskProgress {
    pub(crate) title: Option<String>,
    pub(crate) questions: Vec<CursorQuestion>,
    pub(crate) index: usize,
    pub(crate) answers: Vec<(String, String)>,
}

pub(crate) fn parse_cursor_ask(params: &RawValue) -> Result<CursorAsk, String> {
    let value = json_object(params)?;
    let questions = value
        .get("questions")
        .and_then(Value::as_array)
        .ok_or_else(|| "cursor/ask_question has no questions".to_owned())?
        .iter()
        .map(parse_question)
        .collect::<Result<Vec<_>, _>>()?;
    if questions.is_empty() {
        return Err("cursor/ask_question has no questions".to_owned());
    }
    Ok(CursorAsk {
        tool_call_id: required_string(&value, "toolCallId")?,
        title: optional_string(&value, "title"),
        questions,
    })
}

pub(crate) fn parse_cursor_plan(params: &RawValue) -> Result<CursorPlanRequest, String> {
    let value = json_object(params)?;
    let mut todos = value
        .get("todos")
        .and_then(Value::as_array)
        .map(|todos| todos.iter().map(parse_todo).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    if let Some(phases) = value.get("phases").and_then(Value::as_array) {
        for phase in phases {
            let name = phase.get("name").and_then(Value::as_str).unwrap_or("");
            let phase_todos = phase
                .get("todos")
                .and_then(Value::as_array)
                .map(|todos| todos.iter().map(parse_todo).collect::<Result<Vec<_>, _>>())
                .transpose()?
                .unwrap_or_default();
            for mut todo in phase_todos {
                if !name.is_empty() {
                    todo.content = format!("{name}: {}", todo.content);
                }
                todos.push(todo);
            }
        }
    }
    Ok(CursorPlanRequest {
        name: optional_string(&value, "name"),
        overview: optional_string(&value, "overview"),
        plan: required_string(&value, "plan")?,
        todos,
    })
}

pub(crate) fn parse_cursor_todos(params: &RawValue) -> Result<CursorTodoUpdate, String> {
    let value = json_object(params)?;
    let todos = value
        .get("todos")
        .and_then(Value::as_array)
        .ok_or_else(|| "cursor/update_todos has no todos".to_owned())?
        .iter()
        .map(parse_todo)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CursorTodoUpdate {
        todos,
        merge: value.get("merge").and_then(Value::as_bool).unwrap_or(false),
    })
}

pub(crate) fn parse_cursor_task(params: &RawValue) -> Result<CursorTaskNotice, String> {
    let value = json_object(params)?;
    let subagent_type = value.get("subagentType").cloned().unwrap_or(Value::Null);
    Ok(CursorTaskNotice {
        tool_call_id: required_string(&value, "toolCallId")?,
        description: required_string(&value, "description")?,
        prompt: required_string(&value, "prompt")?,
        subagent_type: format_subagent_type(&subagent_type),
        agent_id: optional_string(&value, "agentId"),
        duration_ms: value.get("durationMs").and_then(Value::as_u64),
    })
}

pub(crate) fn parse_cursor_image(params: &RawValue) -> Result<CursorImageNotice, String> {
    let value = json_object(params)?;
    Ok(CursorImageNotice {
        description: required_string(&value, "description")?,
        file_path: optional_string(&value, "filePath"),
    })
}

fn json_object(params: &RawValue) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(params.get()).map_err(|error| error.to_string())? {
        Value::Object(object) => Ok(object),
        _ => Err("cursor extension params must be an object".to_owned()),
    }
}

fn required_string(object: &Map<String, Value>, key: &str) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing {key}"))
}

fn optional_string(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn parse_question(value: &Value) -> Result<CursorQuestion, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "cursor question must be an object".to_owned())?;
    let options = object
        .get("options")
        .and_then(Value::as_array)
        .ok_or_else(|| "cursor question has no options".to_owned())?
        .iter()
        .map(|option| {
            let option = option
                .as_object()
                .ok_or_else(|| "cursor option must be an object".to_owned())?;
            Ok(CursorOption {
                id: required_string(option, "id")?,
                label: required_string(option, "label")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(CursorQuestion {
        id: required_string(object, "id")?,
        prompt: required_string(object, "prompt")?,
        options,
        allow_multiple: object
            .get("allowMultiple")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn parse_todo(value: &Value) -> Result<CursorTodo, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "cursor todo must be an object".to_owned())?;
    Ok(CursorTodo {
        id: required_string(object, "id")?,
        content: required_string(object, "content")?,
        status: required_string(object, "status")?,
    })
}

pub(crate) fn cursor_plan_entries(todos: &[CursorTodo]) -> Vec<PlanEntry> {
    todos
        .iter()
        .map(|todo| {
            let mut meta = Map::new();
            meta.insert("cursorTodoId".to_owned(), Value::String(todo.id.clone()));
            PlanEntry::new(
                todo.content.clone(),
                PlanEntryPriority::Medium,
                cursor_plan_status(&todo.status),
            )
            .meta(meta)
        })
        .collect()
}

pub(crate) fn cursor_task_region_id(task: &CursorTaskNotice) -> String {
    let id = task
        .agent_id
        .clone()
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| task.tool_call_id.clone());
    format!("cursor:{id}")
}

pub(crate) fn cursor_plan_message(
    name: Option<&str>,
    overview: Option<&str>,
    plan: &str,
) -> String {
    let mut lines = Vec::new();
    if let Some(name) = name.filter(|name| !name.is_empty()) {
        lines.push(name.to_owned());
    }
    if let Some(overview) = overview.filter(|overview| !overview.is_empty()) {
        lines.push(overview.to_owned());
    }
    if !plan.is_empty() {
        lines.push(plan.to_owned());
    }
    if lines.is_empty() {
        "Cursor plan".to_owned()
    } else {
        lines.join("\n\n")
    }
}

pub(crate) fn cursor_image_message(notice: &CursorImageNotice) -> String {
    match notice.file_path.as_deref() {
        Some(path) if !path.is_empty() => {
            format!("Generated image: {} ({path})", notice.description)
        }
        _ => format!("Generated image: {}", notice.description),
    }
}

pub(crate) fn answered_outcome(answers: &[(String, String)]) -> Value {
    Value::Object(Map::from_iter([(
        "outcome".to_owned(),
        Value::Object(Map::from_iter([
            ("outcome".to_owned(), Value::String("answered".to_owned())),
            (
                "answers".to_owned(),
                Value::Array(
                    answers
                        .iter()
                        .map(|(question_id, option_id)| {
                            Value::Object(Map::from_iter([
                                ("questionId".to_owned(), Value::String(question_id.clone())),
                                (
                                    "selectedOptionIds".to_owned(),
                                    Value::Array(vec![Value::String(option_id.clone())]),
                                ),
                            ]))
                        })
                        .collect(),
                ),
            ),
        ])),
    )]))
}

pub(crate) fn named_outcome(outcome: &str) -> Value {
    Value::Object(Map::from_iter([(
        "outcome".to_owned(),
        Value::Object(Map::from_iter([(
            "outcome".to_owned(),
            Value::String(outcome.to_owned()),
        )])),
    )]))
}

pub(crate) fn plan_outcome(accepted: bool) -> Value {
    if accepted {
        Value::Object(Map::from_iter([(
            "outcome".to_owned(),
            Value::Object(Map::from_iter([(
                "outcome".to_owned(),
                Value::String("accepted".to_owned()),
            )])),
        )]))
    } else {
        Value::Object(Map::from_iter([(
            "outcome".to_owned(),
            Value::Object(Map::from_iter([(
                "outcome".to_owned(),
                Value::String("rejected".to_owned()),
            )])),
        )]))
    }
}

pub(crate) fn immediate_cursor_response(
    method: &str,
    params: &RawValue,
) -> Result<ExtResponse, Error> {
    let value = match method {
        CURSOR_UPDATE_TODOS => parse_cursor_todos(params)
            .map(|update| {
                Value::Object(Map::from_iter([(
                    "outcome".to_owned(),
                    Value::Object(Map::from_iter([
                        ("outcome".to_owned(), Value::String("accepted".to_owned())),
                        (
                            "todos".to_owned(),
                            Value::Array(
                                update
                                    .todos
                                    .into_iter()
                                    .map(|todo| {
                                        Value::Object(Map::from_iter([
                                            ("id".to_owned(), Value::String(todo.id)),
                                            ("content".to_owned(), Value::String(todo.content)),
                                            ("status".to_owned(), Value::String(todo.status)),
                                        ]))
                                    })
                                    .collect(),
                            ),
                        ),
                    ])),
                )]))
            })
            .unwrap_or_else(|_| named_outcome("rejected")),
        CURSOR_TASK => parse_cursor_task(params)
            .map(|task| {
                let mut outcome =
                    Map::from_iter([("outcome".to_owned(), Value::String("completed".to_owned()))]);
                if let Some(agent_id) = task.agent_id {
                    outcome.insert("agentId".to_owned(), Value::String(agent_id));
                }
                if let Some(duration_ms) = task.duration_ms {
                    outcome.insert("durationMs".to_owned(), Value::from(duration_ms));
                }
                Value::Object(Map::from_iter([(
                    "outcome".to_owned(),
                    Value::Object(outcome),
                )]))
            })
            .unwrap_or_else(|_| named_outcome("rejected")),
        CURSOR_GENERATE_IMAGE => parse_cursor_image(params)
            .ok()
            .and_then(|notice| notice.file_path)
            .filter(|path| !path.is_empty())
            .map(|file_path| {
                Value::Object(Map::from_iter([(
                    "outcome".to_owned(),
                    Value::Object(Map::from_iter([
                        ("outcome".to_owned(), Value::String("generated".to_owned())),
                        ("filePath".to_owned(), Value::String(file_path)),
                    ])),
                )]))
            })
            .unwrap_or_else(|| named_outcome("rejected")),
        _ => named_outcome("cancelled"),
    };
    ext_response(value)
}

pub(crate) fn ext_response(value: Value) -> Result<ExtResponse, Error> {
    let raw = RawValue::from_string(value.to_string())
        .map_err(|error| Error::internal_error().data(error.to_string()))?;
    Ok(ExtResponse::new(raw.into()))
}

fn format_subagent_type(value: &Value) -> String {
    match value {
        Value::String(kind) => kind.clone(),
        Value::Object(object) => object
            .get("custom")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| "custom".to_owned()),
        _ => "unspecified".to_owned(),
    }
}

fn cursor_plan_status(status: &str) -> PlanEntryStatus {
    match status {
        "in_progress" => PlanEntryStatus::InProgress,
        "completed" | "cancelled" => PlanEntryStatus::Completed,
        _ => PlanEntryStatus::Pending,
    }
}
