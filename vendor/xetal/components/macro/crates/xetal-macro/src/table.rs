//! The macros a file can call (MC18, MC19): the system macros, called
//! unprefixed, and those of the macro libraries it imports, under their
//! alias. A call is appended to its macro library's program and run by
//! [`Libraries::run_macro`].

use xetal_base::Diagnostic;
use xetal_expand::{MacroCall, Macros};
use xetal_lookup::{Libraries, MacroRun};

use crate::macros::{MacroAliases, MacroLib};

pub(crate) struct Table<'a> {
    pub macros: &'a MacroAliases,
    pub system: Option<&'a MacroLib>,
    pub libs: &'a dyn Libraries,
    /// The file the calls are written in: its name and text.
    pub file: (&'a str, &'a str),
}

impl Macros for Table<'_> {
    fn run(&self, call: &MacroCall) -> Result<String, Diagnostic> {
        let lib = self.library(call)?;
        let side = |t: Option<&str>| t.map_or("@".to_string(), quoted);
        let hidden = format!("{}:{}", lib.own, call.name);
        let run = MacroRun {
            line: format!("{} {hidden} {}", side(call.left), side(call.right)),
            hidden,
            written: written(call),
            statement: call.statement,
            texts: (call.left.is_some(), call.right.is_some()),
            file: self.file.0.to_string(),
            row: self.file.1[..call.at.min(self.file.1.len())]
                .matches('\n')
                .count()
                + 1,
        };
        self.libs.run_macro(&lib.sources, &run)
    }
}

impl Table<'_> {
    /// The macro library `call` names: the system macros, or the one
    /// imported under its alias; it must define the macro.
    fn library(&self, call: &MacroCall) -> Result<&MacroLib, Diagnostic> {
        let defines = |lib: &MacroLib| lib.exports.iter().any(|e| e == call.name);
        let Some(ns) = call.ns else {
            return match self.system {
                Some(system) if defines(system) => Ok(system),
                system => Err(unknown_system(call.name, system)),
            };
        };
        let Some(lib) = self.macros.get(ns) else {
            let message = format!(
                "there is no macro {ns}:{}: no macro library is imported as {ns}: (\"{ns}:\" u_se< \"Name\" finds Name.xtlm)",
                call.name
            );
            return Err(Diagnostic::new("unknown-macro", message));
        };
        if defines(lib) {
            return Ok(lib);
        }
        let message = format!(
            "{ns}:{} is not defined by that macro library; it defines {}",
            call.name,
            lib.exports.join(", ")
        );
        Err(Diagnostic::new("not-exported", message))
    }
}

/// An unprefixed macro that is not a system macro.
fn unknown_system(name: &str, system: Option<&MacroLib>) -> Diagnostic {
    let mut names = vec!["u_se<".to_string()];
    names.extend(system.iter().flat_map(|s| s.exports.iter().cloned()));
    let last = names.pop().unwrap_or_default();
    let list = match names.is_empty() {
        true => last,
        false => format!("{} and {last}", names.join(", ")),
    };
    let message = format!("there is no macro {name}; the system macros are {list}");
    Diagnostic::new("unknown-macro", message)
}

/// The macro as the call writes it.
fn written(call: &MacroCall) -> String {
    match call.ns {
        Some(ns) => format!("{ns}:{}", call.name),
        None => call.name.to_string(),
    }
}

/// `text` as a string literal.
fn quoted(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}
