//! Algorithm W over Core programs: top-level items, late-bound module
//! definitions, top-level defaulting (T5).

use std::collections::HashMap;

use xetal_base::{Diagnostic, NodeId, Span};
use xetal_core::{Expr, Item, Program};

use xetal_elab::Dicts;
use xetal_ty::Unifier;

use crate::record::Record;
use xetal_ty::mono;
use xetal_ty::{Scheme, Type};

pub(crate) enum Global {
    Defined(Scheme),
    /// Used before its definition: one monomorphic type until then.
    Pending(Type, Span),
    /// Defined while a forward reference is pending: monomorphic until
    /// its binding group closes (`line` is its output line).
    Open {
        ty: Type,
        value: Expr,
        line: usize,
    },
}

pub(crate) struct Infer {
    pub u: Unifier,
    pub globals: HashMap<String, Global>,
    /// Lexical bindings, innermost last, with the value node of the
    /// binding (none for a lambda parameter).
    pub env: Vec<(String, Scheme, Option<NodeId>)>,
    /// Output lines: an optional name and its type, shown at the end.
    pub lines: Vec<(Option<String>, Scheme)>,
    /// Facts for elaboration (T6).
    pub rec: Record,
    /// The last type variable made before the current top-level item.
    pub mark: u32,
}

/// Infer a whole program. Returns one line per item (`name : type` for
/// bindings and definitions, the type for expressions) and the facts
/// elaboration needs.
pub fn infer_program(program: &Program) -> Result<(Vec<String>, Dicts), Diagnostic> {
    let mut inf = Infer {
        u: Unifier::default(),
        globals: HashMap::new(),
        env: Vec::new(),
        lines: Vec::new(),
        rec: Record::default(),
        mark: 0,
    };
    for item in &program.items {
        inf.item(item).map_err(|d| program.annotate(d))?;
    }
    for global in inf.globals.values() {
        if let Global::Pending(_, span) = global {
            return Err(
                Diagnostic::new("undefined-name", "this name is never defined").with_span(*span),
            );
        }
    }
    let lines = inf.lines.iter().map(|(name, scheme)| {
        let scheme = Scheme {
            ty: inf.u.resolve(&scheme.ty),
            ..scheme.clone()
        };
        match name {
            Some(name) => format!("{name} : {scheme}"),
            None => scheme.to_string(),
        }
    });
    Ok((lines.collect(), inf.dicts()))
}

impl Infer {
    fn item(&mut self, item: &Item) -> Result<(), Diagnostic> {
        self.mark = self.u.mark();
        let line = match item {
            Item::Def { name, value } => return self.def(name, value),
            Item::Let { name, rec, value } => {
                let t = self.binding(name, *rec, value)?;
                let (scheme, shown) = match self.condition(&t) {
                    Some(scheme) => (scheme, mono(Type::Bool)),
                    None => {
                        let scheme = self.close(&t, value, true)?;
                        (scheme.clone(), scheme)
                    }
                };
                self.env.push((name.clone(), scheme, Some(value.id)));
                (Some(name.clone()), shown)
            }
            Item::Set { name, value } => {
                let t = self.set(name, value)?;
                (Some(name.clone()), mono(self.u.defaulted(&t)))
            }
            Item::Eval(e) => {
                let t = self.expr(e)?;
                (None, self.close(&t, e, true)?)
            }
        };
        self.lines.push(line);
        Ok(())
    }

    /// A module definition joins the open binding group; the group is
    /// generalized together once no forward reference is pending.
    fn def(&mut self, name: &str, value: &Expr) -> Result<(), Diagnostic> {
        let v = match self.globals.get(name) {
            Some(Global::Pending(t, _)) => t.clone(),
            Some(_) => {
                return Err(Diagnostic::new(
                    "duplicate-definition",
                    format!("{name} is already defined in this file"),
                )
                .with_span(value.span));
            }
            None => self.u.fresh(),
        };
        self.globals
            .insert(name.to_string(), Global::Pending(v.clone(), value.span));
        self.rec.global(name, value.id);
        let t = self.expr(value)?;
        self.u.unify(&v, &t, value.span)?;
        let open = Global::Open {
            ty: t,
            value: value.clone(),
            line: self.lines.len(),
        };
        self.globals.insert(name.to_string(), open);
        self.lines.push((Some(name.to_string()), mono(v)));
        if self
            .globals
            .values()
            .any(|g| matches!(g, Global::Pending(..)))
        {
            return Ok(());
        }
        let names: Vec<String> = self.globals.keys().cloned().collect();
        let group: Vec<_> = names
            .into_iter()
            .filter_map(|n| match self.globals.remove(&n) {
                Some(Global::Open { ty, value, line }) => Some((n, ty, value, line)),
                Some(other) => {
                    self.globals.insert(n, other);
                    None
                }
                None => None,
            })
            .collect();
        for (name, ty, value, line) in group {
            let scheme = self.close(&ty, &value, true)?;
            self.lines[line].1 = scheme.clone();
            self.globals.insert(name, Global::Defined(scheme));
        }
        Ok(())
    }

    /// The type of a binding's value; a recursive binding sees itself.
    pub(crate) fn binding(
        &mut self,
        name: &str,
        rec: bool,
        value: &Expr,
    ) -> Result<Type, Diagnostic> {
        if !rec {
            return self.expr(value);
        }
        let v = self.u.fresh();
        self.env
            .push((name.to_string(), mono(v.clone()), Some(value.id)));
        let t = self.expr(value);
        self.env.pop();
        let t = t?;
        self.u.unify(&v, &t, value.span)?;
        Ok(t)
    }

    pub(crate) fn set(&mut self, name: &str, value: &Expr) -> Result<Type, Diagnostic> {
        let Some((_, scheme, _)) = self.env.iter().rev().find(|(n, ..)| n == name) else {
            return Err(
                Diagnostic::new("undefined-name", format!("{name} is not defined"))
                    .with_span(value.span),
            );
        };
        let current = scheme.ty.clone();
        let t = self.expr(value)?;
        self.u.unify(&current, &t, value.span)?;
        Ok(t)
    }
}
