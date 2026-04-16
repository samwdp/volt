use crate::{
    autocomplete::AutocompleteProviderConfig,
    hover::HoverProviderConfig,
    icon_font::symbols::{cod, md},
};
use editor_plugin_api::{
    AutocompleteProviderItem, HoverProviderTopic, PluginAction, PluginBuffer, PluginBufferSection,
    PluginBufferSectionUpdate, PluginBufferSections, PluginCommand, PluginKeyBinding,
    PluginKeymapScope, PluginPackage, buffer_kinds, plugin_hooks,
};

// ─── Public constants ─────────────────────────────────────────────────────────

pub use buffer_kinds::CALCULATOR as CALCULATOR_KIND;

pub const BUFFER_NAME: &str = "*calculator*";
pub const EVALUATE_HANDLER: &str = "calculator.evaluate-buffer";
pub const EVALUATE_CHORD: &str = "Ctrl+c Ctrl+c";
pub const SWITCH_PANE_CHORD: &str = "Ctrl+Tab";
pub const PROVIDER_CALCULATOR: &str = "calculator";
pub const PROVIDER_LABEL: &str = "Calculator";

const PROVIDER_ICON: &str = md::MD_CALCULATOR;
const AUTOCOMPLETE_ITEM_ICON: &str = md::MD_FUNCTION;
const HOVER_TOPIC_ICON: &str = cod::COD_INFO;

// ─── Package ─────────────────────────────────────────────────────────────────

/// Returns the plugin package for the calculator.
pub fn package() -> PluginPackage {
    PluginPackage::new("calculator", true, "Expression evaluator buffer.")
        .with_commands(vec![
            PluginCommand::new(
                "calculator.open",
                "Open the calculator buffer in the active pane.",
                vec![PluginAction::open_buffer(
                    BUFFER_NAME,
                    CALCULATOR_KIND,
                    None::<&str>,
                )],
            ),
            // Emits the generic plugin.evaluate hook; the host handles the rest.
            // The host reads the active buffer kind and calls
            // UserLibrary::handle_plugin_evaluate("calculator", input).
            PluginCommand::new(
                "calculator.evaluate",
                "Evaluate the calculator input and write results to the output section.",
                vec![PluginAction::emit_hook(
                    plugin_hooks::EVALUATE,
                    None::<&str>,
                )],
            ),
            PluginCommand::new(
                "calculator.switch-pane",
                "Switch focus between the calculator input and output panes.",
                vec![PluginAction::emit_hook(
                    plugin_hooks::SWITCH_PANE,
                    None::<&str>,
                )],
            ),
        ])
        .with_buffers(vec![
            PluginBuffer::new(CALCULATOR_KIND, initial_buffer_lines())
                .with_sections(buffer_sections())
                .with_evaluate_handler(EVALUATE_HANDLER)
                .with_evaluate_target_section("Output")
                .with_key_bindings(vec![
                    PluginKeyBinding::new(
                        EVALUATE_CHORD,
                        "calculator.evaluate",
                        PluginKeymapScope::Workspace,
                    ),
                    PluginKeyBinding::new(
                        SWITCH_PANE_CHORD,
                        "calculator.switch-pane",
                        PluginKeymapScope::Workspace,
                    ),
                ]),
        ])
}

// ─── Initial content ──────────────────────────────────────────────────────────

/// Returns the initial lines placed in a freshly-opened calculator buffer.
pub fn initial_buffer_lines() -> Vec<String> {
    vec![
        format!(
            "# Write expressions below. Press {EVALUATE_CHORD} to evaluate, or {SWITCH_PANE_CHORD} to switch panes."
        ),
        String::new(),
        "a = 1".to_owned(),
        "b = 2".to_owned(),
        "sqrt(a + b)".to_owned(),
    ]
}

/// Returns the split-pane layout used by the calculator buffer.
pub fn buffer_sections() -> PluginBufferSections {
    PluginBufferSections::new(vec![
        PluginBufferSection::new("Input")
            .with_writable(true)
            .with_initial_lines(initial_buffer_lines()),
        PluginBufferSection::new("Output")
            .with_min_lines(1)
            .with_initial_lines(vec!["(press Ctrl+c Ctrl+c to evaluate)".to_owned()])
            .with_update(PluginBufferSectionUpdate::Replace),
    ])
}

pub fn autocomplete_provider() -> AutocompleteProviderConfig {
    AutocompleteProviderConfig::new(
        PROVIDER_CALCULATOR,
        PROVIDER_LABEL,
        PROVIDER_ICON,
        AUTOCOMPLETE_ITEM_ICON,
    )
    .with_buffer_kind(CALCULATOR_KIND)
    .with_items(autocomplete_items())
}

pub fn hover_provider() -> HoverProviderConfig {
    HoverProviderConfig::new(PROVIDER_CALCULATOR, PROVIDER_LABEL, PROVIDER_ICON)
        .with_buffer_kind(CALCULATOR_KIND)
        .with_topics(hover_topics())
}

pub fn autocomplete_items() -> Vec<AutocompleteProviderItem> {
    calculator_symbols()
        .iter()
        .map(|symbol| AutocompleteProviderItem {
            label: symbol.label.to_owned(),
            replacement: symbol.replacement.to_owned(),
            detail: Some(symbol.detail.to_owned()),
            documentation: Some(symbol.documentation.to_owned()),
        })
        .collect()
}

pub fn hover_topics() -> Vec<HoverProviderTopic> {
    calculator_symbols()
        .iter()
        .map(|symbol| HoverProviderTopic {
            token: symbol.replacement.to_owned(),
            lines: hover_lines(symbol),
        })
        .collect()
}

// ─── Evaluator ───────────────────────────────────────────────────────────────

/// Evaluate `input` (newline-separated expression lines) and return the output
/// lines to display in the output section of the calculator buffer.
///
/// Lines beginning with `#` are comments; blank lines are skipped.
/// Lines matching `name = expr` assign a variable.
/// All other lines are evaluated and their result appended to the output.
pub fn evaluate(input: &str) -> Vec<String> {
    let mut env = Env::default();
    let mut output = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match eval_line(line, &mut env) {
            EvalResult::Assignment(name, value) => {
                env.set(&name, value);
                output.push(format!("{name} = {}", format_value(value)));
            }
            EvalResult::Value(value) => {
                output.push(format_value(value));
            }
            EvalResult::Error(msg) => {
                output.push(format!("error: {msg}"));
            }
        }
    }

    if output.is_empty() {
        output.push("(no expressions)".to_owned());
    }
    output
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

struct CalculatorSymbol {
    label: &'static str,
    replacement: &'static str,
    detail: &'static str,
    documentation: &'static str,
}

const CALCULATOR_SYMBOLS: &[CalculatorSymbol] = &[
    CalculatorSymbol {
        label: "sqrt(x)",
        replacement: "sqrt",
        detail: "Square root",
        documentation: "Returns the square root of x.",
    },
    CalculatorSymbol {
        label: "cbrt(x)",
        replacement: "cbrt",
        detail: "Cube root",
        documentation: "Returns the cube root of x.",
    },
    CalculatorSymbol {
        label: "abs(x)",
        replacement: "abs",
        detail: "Absolute value",
        documentation: "Returns the absolute value of x.",
    },
    CalculatorSymbol {
        label: "floor(x)",
        replacement: "floor",
        detail: "Round down",
        documentation: "Rounds x down to the nearest integer.",
    },
    CalculatorSymbol {
        label: "ceil(x)",
        replacement: "ceil",
        detail: "Round up",
        documentation: "Rounds x up to the nearest integer.",
    },
    CalculatorSymbol {
        label: "round(x)",
        replacement: "round",
        detail: "Round to nearest integer",
        documentation: "Rounds x to the nearest integer.",
    },
    CalculatorSymbol {
        label: "sin(x)",
        replacement: "sin",
        detail: "Sine",
        documentation: "Returns the sine of x in radians.",
    },
    CalculatorSymbol {
        label: "cos(x)",
        replacement: "cos",
        detail: "Cosine",
        documentation: "Returns the cosine of x in radians.",
    },
    CalculatorSymbol {
        label: "tan(x)",
        replacement: "tan",
        detail: "Tangent",
        documentation: "Returns the tangent of x in radians.",
    },
    CalculatorSymbol {
        label: "asin(x)",
        replacement: "asin",
        detail: "Arcsine",
        documentation: "Returns the arcsine of x in radians.",
    },
    CalculatorSymbol {
        label: "acos(x)",
        replacement: "acos",
        detail: "Arccosine",
        documentation: "Returns the arccosine of x in radians.",
    },
    CalculatorSymbol {
        label: "atan(x)",
        replacement: "atan",
        detail: "Arctangent",
        documentation: "Returns the arctangent of x in radians.",
    },
    CalculatorSymbol {
        label: "atan2(y, x)",
        replacement: "atan2",
        detail: "Two-argument arctangent",
        documentation: "Returns the angle for the point (x, y) in radians.",
    },
    CalculatorSymbol {
        label: "ln(x)",
        replacement: "ln",
        detail: "Natural logarithm",
        documentation: "Returns the natural logarithm of x.",
    },
    CalculatorSymbol {
        label: "log(x)",
        replacement: "log",
        detail: "Base-10 logarithm",
        documentation: "Returns the base-10 logarithm of x.",
    },
    CalculatorSymbol {
        label: "log2(x)",
        replacement: "log2",
        detail: "Base-2 logarithm",
        documentation: "Returns the base-2 logarithm of x.",
    },
    CalculatorSymbol {
        label: "log10(x)",
        replacement: "log10",
        detail: "Base-10 logarithm",
        documentation: "Returns the base-10 logarithm of x.",
    },
    CalculatorSymbol {
        label: "exp(x)",
        replacement: "exp",
        detail: "Exponential",
        documentation: "Returns e raised to the power of x.",
    },
    CalculatorSymbol {
        label: "pow(base, exponent)",
        replacement: "pow",
        detail: "Power",
        documentation: "Returns base raised to exponent.",
    },
    CalculatorSymbol {
        label: "min(a, b)",
        replacement: "min",
        detail: "Minimum",
        documentation: "Returns the smaller of a and b.",
    },
    CalculatorSymbol {
        label: "max(a, b)",
        replacement: "max",
        detail: "Maximum",
        documentation: "Returns the larger of a and b.",
    },
    CalculatorSymbol {
        label: "pi",
        replacement: "pi",
        detail: "Circle constant π",
        documentation: "Built-in constant for π.",
    },
    CalculatorSymbol {
        label: "e",
        replacement: "e",
        detail: "Euler's number",
        documentation: "Built-in constant for Euler's number.",
    },
    CalculatorSymbol {
        label: "tau",
        replacement: "tau",
        detail: "Circle constant τ",
        documentation: "Built-in constant for τ (2π).",
    },
    CalculatorSymbol {
        label: "inf",
        replacement: "inf",
        detail: "Infinity",
        documentation: "Built-in constant for positive infinity.",
    },
    CalculatorSymbol {
        label: "nan",
        replacement: "nan",
        detail: "Not a number",
        documentation: "Built-in constant for NaN.",
    },
];

fn calculator_symbols() -> &'static [CalculatorSymbol] {
    CALCULATOR_SYMBOLS
}

fn hover_lines(symbol: &CalculatorSymbol) -> Vec<String> {
    vec![
        format!("{HOVER_TOPIC_ICON} {}", symbol.label),
        symbol.detail.to_owned(),
        String::new(),
        symbol.documentation.to_owned(),
    ]
}

fn format_value(v: f64) -> String {
    if v.is_nan() {
        return "nan".to_owned();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "inf".to_owned()
        } else {
            "-inf".to_owned()
        };
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    format!("{v}")
}

enum EvalResult {
    Assignment(String, f64),
    Value(f64),
    Error(String),
}

fn eval_line(line: &str, env: &mut Env) -> EvalResult {
    if let Some((lhs, rhs)) = split_assignment(line)
        && is_valid_ident(lhs)
    {
        match Parser::new(rhs, env).parse() {
            Ok(value) => return EvalResult::Assignment(lhs.to_owned(), value),
            Err(e) => return EvalResult::Error(e),
        }
    }
    match Parser::new(line, env).parse() {
        Ok(value) => EvalResult::Value(value),
        Err(e) => EvalResult::Error(e),
    }
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let bytes = line.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == b'='
            && !matches!(bytes[i - 1], b'<' | b'>' | b'!' | b'=')
            && (i + 1 >= bytes.len() || bytes[i + 1] != b'=')
        {
            return Some((line[..i].trim_end(), line[i + 1..].trim_start()));
        }
    }
    None
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

// ─── Variable environment ─────────────────────────────────────────────────────

#[derive(Default)]
struct Env {
    vars: Vec<(String, f64)>,
}

impl Env {
    fn get(&self, name: &str) -> Option<f64> {
        self.vars
            .iter()
            .rev()
            .find_map(|(k, v)| (k == name).then_some(*v))
    }

    fn set(&mut self, name: &str, value: f64) {
        if let Some(entry) = self.vars.iter_mut().find(|(k, _)| k == name) {
            entry.1 = value;
        } else {
            self.vars.push((name.to_owned(), value));
        }
    }
}

// ─── Tokenizer ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Eof,
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Ok(Token::Eof);
        }
        let b = self.input.as_bytes()[self.pos];
        let token = match b {
            b'+' => {
                self.pos += 1;
                Token::Plus
            }
            b'-' => {
                self.pos += 1;
                Token::Minus
            }
            b'*' => {
                self.pos += 1;
                Token::Star
            }
            b'/' => {
                self.pos += 1;
                Token::Slash
            }
            b'%' => {
                self.pos += 1;
                Token::Percent
            }
            b'^' => {
                self.pos += 1;
                Token::Caret
            }
            b'(' => {
                self.pos += 1;
                Token::LParen
            }
            b')' => {
                self.pos += 1;
                Token::RParen
            }
            b',' => {
                self.pos += 1;
                Token::Comma
            }
            b'0'..=b'9' | b'.' => self.read_number()?,
            _ if (b as char).is_alphabetic() || b == b'_' => self.read_ident(),
            other => return Err(format!("unexpected character `{}`", other as char)),
        };
        Ok(token)
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input.as_bytes()[self.pos].is_ascii_digit()
                || self.input.as_bytes()[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos < self.input.len() && matches!(self.input.as_bytes()[self.pos], b'e' | b'E') {
            self.pos += 1;
            if self.pos < self.input.len() && matches!(self.input.as_bytes()[self.pos], b'+' | b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let s = &self.input[start..self.pos];
        s.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| format!("invalid number `{s}`"))
    }

    fn read_ident(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::Ident(self.input[start..self.pos].to_owned())
    }
}

// ─── Recursive descent parser ─────────────────────────────────────────────────

struct Parser<'a, 'b> {
    lexer: Lexer<'a>,
    current: Token,
    env: &'b Env,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(input: &'a str, env: &'b Env) -> Self {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token().unwrap_or(Token::Eof);
        Self {
            lexer,
            current,
            env,
        }
    }

    fn advance(&mut self) -> Result<(), String> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn parse(&mut self) -> Result<f64, String> {
        let result = self.expr()?;
        if self.current != Token::Eof {
            return Err(format!(
                "unexpected token after expression: {:?}",
                self.remaining_input()
            ));
        }
        Ok(result)
    }

    fn remaining_input(&self) -> &str {
        self.lexer.remaining()
    }

    fn expr(&mut self) -> Result<f64, String> {
        self.additive()
    }

    fn additive(&mut self) -> Result<f64, String> {
        let mut left = self.multiplicative()?;
        loop {
            match self.current {
                Token::Plus => {
                    self.advance()?;
                    left += self.multiplicative()?;
                }
                Token::Minus => {
                    self.advance()?;
                    left -= self.multiplicative()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<f64, String> {
        let mut left = self.unary()?;
        loop {
            match self.current {
                Token::Star => {
                    self.advance()?;
                    left *= self.unary()?;
                }
                Token::Slash => {
                    self.advance()?;
                    let right = self.unary()?;
                    if right == 0.0 {
                        return Err("division by zero".to_owned());
                    }
                    left /= right;
                }
                Token::Percent => {
                    self.advance()?;
                    let right = self.unary()?;
                    if right == 0.0 {
                        return Err("modulo by zero".to_owned());
                    }
                    left %= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<f64, String> {
        if self.current == Token::Minus {
            self.advance()?;
            return Ok(-self.unary()?);
        }
        self.power()
    }

    fn power(&mut self) -> Result<f64, String> {
        let base = self.primary()?;
        if self.current == Token::Caret {
            self.advance()?;
            let exp = self.unary()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<f64, String> {
        match self.current.clone() {
            Token::Number(n) => {
                self.advance()?;
                Ok(n)
            }
            Token::Ident(name) => {
                self.advance()?;
                if self.current == Token::LParen {
                    self.advance()?;
                    let args = self.arg_list()?;
                    if self.current != Token::RParen {
                        return Err(format!("expected `)` after arguments of `{name}`"));
                    }
                    self.advance()?;
                    call_function(&name, &args)
                } else {
                    self.lookup_constant_or_var(&name)
                }
            }
            Token::LParen => {
                self.advance()?;
                let value = self.expr()?;
                if self.current != Token::RParen {
                    return Err("expected closing `)`".to_owned());
                }
                self.advance()?;
                Ok(value)
            }
            Token::Eof => Err("unexpected end of expression".to_owned()),
            other => Err(format!("unexpected token `{other:?}`")),
        }
    }

    fn arg_list(&mut self) -> Result<Vec<f64>, String> {
        let mut args = Vec::new();
        if self.current == Token::RParen {
            return Ok(args);
        }
        args.push(self.expr()?);
        while self.current == Token::Comma {
            self.advance()?;
            args.push(self.expr()?);
        }
        Ok(args)
    }

    fn lookup_constant_or_var(&self, name: &str) -> Result<f64, String> {
        match name {
            "pi" | "PI" => Ok(std::f64::consts::PI),
            "e" | "E" => Ok(std::f64::consts::E),
            "tau" | "TAU" => Ok(std::f64::consts::TAU),
            "inf" | "INF" | "infinity" => Ok(f64::INFINITY),
            "nan" | "NAN" => Ok(f64::NAN),
            _ => self
                .env
                .get(name)
                .ok_or_else(|| format!("undefined variable `{name}`")),
        }
    }
}

fn call_function(name: &str, args: &[f64]) -> Result<f64, String> {
    let one = |args: &[f64]| -> Result<f64, String> {
        if args.len() == 1 {
            Ok(args[0])
        } else {
            Err(format!("`{name}` expects 1 argument, got {}", args.len()))
        }
    };
    let two = |args: &[f64]| -> Result<(f64, f64), String> {
        if args.len() == 2 {
            Ok((args[0], args[1]))
        } else {
            Err(format!("`{name}` expects 2 arguments, got {}", args.len()))
        }
    };
    match name {
        "sqrt" => Ok(one(args)?.sqrt()),
        "cbrt" => Ok(one(args)?.cbrt()),
        "abs" => Ok(one(args)?.abs()),
        "floor" => Ok(one(args)?.floor()),
        "ceil" => Ok(one(args)?.ceil()),
        "round" => Ok(one(args)?.round()),
        "sin" => Ok(one(args)?.sin()),
        "cos" => Ok(one(args)?.cos()),
        "tan" => Ok(one(args)?.tan()),
        "asin" => Ok(one(args)?.asin()),
        "acos" => Ok(one(args)?.acos()),
        "atan" => Ok(one(args)?.atan()),
        "atan2" => {
            let (y, x) = two(args)?;
            Ok(y.atan2(x))
        }
        "ln" => Ok(one(args)?.ln()),
        "log" => Ok(one(args)?.log10()),
        "log2" => Ok(one(args)?.log2()),
        "log10" => Ok(one(args)?.log10()),
        "exp" => Ok(one(args)?.exp()),
        "pow" => {
            let (b, e) = two(args)?;
            Ok(b.powf(e))
        }
        "min" => {
            let (a, b) = two(args)?;
            Ok(a.min(b))
        }
        "max" => {
            let (a, b) = two(args)?;
            Ok(a.max(b))
        }
        _ => Err(format!("unknown function `{name}`")),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_one(expr: &str) -> String {
        evaluate(expr).into_iter().next().unwrap_or_default()
    }

    #[test]
    fn calculator_package_exports_open_and_evaluate_commands() {
        let pkg = package();
        assert!(pkg.commands().iter().any(|c| c.name() == "calculator.open"));
        assert!(
            pkg.commands()
                .iter()
                .any(|c| c.name() == "calculator.evaluate")
        );
    }

    #[test]
    fn calculator_evaluate_command_emits_generic_plugin_evaluate_hook() {
        let pkg = package();
        let cmd = pkg
            .commands()
            .iter()
            .find(|c| c.name() == "calculator.evaluate")
            .expect("calculator.evaluate command must exist");
        assert!(
            cmd.actions().iter().any(|a| a
                .hook()
                .is_some_and(|h| h.hook_name() == plugin_hooks::EVALUATE)),
            "calculator.evaluate must emit the generic plugin.evaluate hook"
        );
    }

    #[test]
    fn calculator_package_binds_ctrl_c_ctrl_c() {
        let pkg = package();
        let buffer = pkg
            .buffer(CALCULATOR_KIND)
            .expect("calculator package should declare its plugin buffer");
        assert!(
            buffer
                .key_bindings()
                .iter()
                .any(|kb| kb.chord() == EVALUATE_CHORD)
        );
    }

    #[test]
    fn calculator_package_binds_ctrl_tab_to_switch_panes() {
        let pkg = package();
        let buffer = pkg
            .buffer(CALCULATOR_KIND)
            .expect("calculator package should declare its plugin buffer");
        assert!(
            buffer
                .key_bindings()
                .iter()
                .any(|kb| kb.chord() == SWITCH_PANE_CHORD
                    && kb.command_name() == "calculator.switch-pane")
        );
    }

    #[test]
    fn calculator_package_has_no_hook_declarations() {
        // plugin.evaluate is owned by the host; plugins must not re-declare it.
        let pkg = package();
        assert!(
            pkg.hook_declarations().is_empty(),
            "user plugins must not declare host-owned hooks"
        );
    }

    #[test]
    fn evaluator_handles_literals() {
        assert_eq!(eval_one("42"), "42");
        assert_eq!(eval_one("3.14"), "3.14");
    }

    #[test]
    fn evaluator_handles_arithmetic() {
        assert_eq!(eval_one("2 + 3"), "5");
        assert_eq!(eval_one("10 - 3"), "7");
        assert_eq!(eval_one("4 * 5"), "20");
        assert_eq!(eval_one("10 / 4"), "2.5");
    }

    #[test]
    fn evaluator_handles_power() {
        assert_eq!(eval_one("2 ^ 10"), "1024");
    }

    #[test]
    fn evaluator_handles_variables() {
        let output = evaluate("a = 3\nb = 4\na + b");
        assert_eq!(output, vec!["a = 3", "b = 4", "7"]);
    }

    #[test]
    fn evaluator_handles_sqrt() {
        assert_eq!(eval_one("sqrt(4)"), "2");
        assert_eq!(eval_one("sqrt(2)"), "1.4142135623730951");
    }

    #[test]
    fn evaluator_handles_sqrt_of_sum_of_two_vars() {
        let output = evaluate("a = 1\nb = 2\nsqrt((a + b))");
        assert_eq!(
            output.last().expect("output should not be empty"),
            "1.7320508075688772"
        );
    }

    #[test]
    fn evaluator_handles_pi_constant() {
        assert_eq!(eval_one("pi"), format_value(std::f64::consts::PI));
    }

    #[test]
    fn evaluator_handles_comments() {
        let output = evaluate("# this is a comment\n1 + 1");
        assert_eq!(output, vec!["2"]);
    }

    #[test]
    fn evaluator_handles_blank_lines() {
        let output = evaluate("1\n\n2");
        assert_eq!(output, vec!["1", "2"]);
    }

    #[test]
    fn evaluator_reports_undefined_variable() {
        assert!(evaluate("x + 1")[0].starts_with("error:"));
    }

    #[test]
    fn evaluator_reports_unknown_function() {
        assert!(evaluate("foo(1)")[0].starts_with("error:"));
    }

    #[test]
    fn evaluator_returns_no_expressions_for_empty_input() {
        assert_eq!(evaluate(""), vec!["(no expressions)"]);
    }

    #[test]
    fn initial_buffer_lines_only_seed_input_examples() {
        let lines = initial_buffer_lines();
        assert!(lines.iter().all(|line| !line.starts_with("─── Output")));
        assert!(lines.iter().any(|line| line == "sqrt(a + b)"));
    }

    #[test]
    fn calculator_buffer_sections_start_with_single_output_row() {
        let sections = buffer_sections();
        let names = sections
            .items()
            .iter()
            .map(|section| section.name())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Input", "Output"]);
        assert!(sections.items()[0].writable());
        assert_eq!(sections.items()[1].min_lines(), Some(1));
        assert_eq!(
            sections.items()[1]
                .initial_lines()
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
            vec!["(press Ctrl+c Ctrl+c to evaluate)"]
        );
    }

    #[test]
    fn calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers() {
        let provider = autocomplete_provider();
        assert_eq!(provider.id, PROVIDER_CALCULATOR);
        assert_eq!(provider.buffer_kind.as_deref(), Some(CALCULATOR_KIND));
        assert!(provider.items.iter().any(|item| item.replacement == "sqrt"
            && item.label == "sqrt(x)"
            && item.detail.as_deref() == Some("Square root")));
        assert!(
            provider
                .items
                .iter()
                .any(|item| item.replacement == "pi" && item.documentation.is_some())
        );
    }

    #[test]
    fn calculator_hover_provider_exports_function_and_constant_topics() {
        let provider = hover_provider();
        assert_eq!(provider.id, PROVIDER_CALCULATOR);
        assert_eq!(provider.buffer_kind.as_deref(), Some(CALCULATOR_KIND));
        assert!(provider.topics.iter().any(|topic| {
            topic.token == "atan2"
                && topic
                    .lines
                    .iter()
                    .any(|line| line.contains("Two-argument arctangent"))
        }));
        assert!(provider.topics.iter().any(|topic| {
            topic.token == "tau"
                && topic
                    .lines
                    .iter()
                    .any(|line| line.contains("Built-in constant for τ"))
        }));
    }

    #[test]
    fn calculator_switch_pane_command_emits_generic_switch_hook() {
        let pkg = package();
        let cmd = pkg
            .commands()
            .iter()
            .find(|c| c.name() == "calculator.switch-pane")
            .expect("calculator.switch-pane command must exist");
        assert!(
            cmd.actions().iter().any(|a| a
                .hook()
                .is_some_and(|h| h.hook_name() == plugin_hooks::SWITCH_PANE)),
            "calculator.switch-pane must emit the generic plugin.switch-pane hook"
        );
    }

    #[test]
    fn calculator_package_declares_its_buffer_through_package_metadata() {
        let pkg = package();
        let buffer = pkg
            .buffer(CALCULATOR_KIND)
            .expect("calculator package should declare its plugin buffer");
        assert_eq!(buffer.kind(), CALCULATOR_KIND);
        assert_eq!(buffer.evaluate_handler(), Some(EVALUATE_HANDLER));
        assert_eq!(buffer.evaluate_target_section(), Some("Output"));
        assert_eq!(
            buffer
                .key_bindings()
                .iter()
                .map(|binding| binding.chord())
                .collect::<Vec<_>>(),
            vec![EVALUATE_CHORD, SWITCH_PANE_CHORD]
        );
        assert_eq!(
            buffer.sections().map(|sections| {
                sections
                    .items()
                    .iter()
                    .map(|section| section.name())
                    .collect::<Vec<_>>()
            }),
            Some(vec!["Input", "Output"])
        );
    }
}
