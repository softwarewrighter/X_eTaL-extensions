//! The elaboration walk over a Core program.

use xetal_core::{Expr, Item, Kind, Program};
use xetal_lex::Number;

use crate::Dicts;
use crate::build::{Scope, app, lam, literal, typed_result, zero};

struct Walker<'a> {
    dicts: &'a Dicts,
    scope: Scope,
    /// Hidden parameters made so far (names `#n1`, `#n2`, ...).
    made: u32,
}

/// Rewrite `program` with the number-type facts in `dicts`.
pub fn elaborate(program: &mut Program, dicts: &Dicts) {
    let mut w = Walker {
        dicts,
        scope: Vec::new(),
        made: 0,
    };
    for item in &mut program.items {
        match item {
            Item::Def { value, .. } | Item::Let { value, .. } => w.binding(value),
            Item::Set { value, .. } | Item::Eval(value) => w.expr(value),
        }
    }
}

impl Walker<'_> {
    /// A bound value: generalized over `Num` variables, it takes one
    /// hidden parameter per variable.
    fn binding(&mut self, value: &mut Expr) {
        let vars = self
            .dicts
            .params
            .get(&value.id)
            .cloned()
            .unwrap_or_default();
        let names: Vec<String> = vars
            .iter()
            .map(|_| {
                self.made += 1;
                format!("#n{}", self.made)
            })
            .collect();
        let outer = self.scope.len();
        self.scope
            .extend(vars.into_iter().zip(names.iter().cloned()));
        self.expr(value);
        self.scope.truncate(outer);
        for name in names.into_iter().rev() {
            *value = lam(name, value.clone());
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        match &mut e.kind {
            Kind::Lit(Number::Int(n)) => {
                let n = *n;
                if let Some(t) = self.dicts.lits.get(&e.id) {
                    *e = literal(e, n, t, &self.scope);
                }
            }
            Kind::Var(_) | Kind::Global(_) => self.pass(e),
            Kind::Axes { f, .. } if self.dicts.prims.contains_key(&f.id) => self.subscripted(e),
            Kind::Prim(_) => {
                if let Some((t, arity)) = self.dicts.prims.get(&e.id) {
                    *e = typed_result(e, t, *arity, &self.scope);
                }
            }
            Kind::Let { value, body, .. } => {
                self.binding(value);
                self.expr(body);
            }
            _ => self.children(e),
        }
    }

    /// A use of a generalized binding passes its number types.
    fn pass(&mut self, e: &mut Expr) {
        for t in self.dicts.args.get(&e.id).into_iter().flatten() {
            let z = zero(e, t, &self.scope);
            *e = app(e.clone(), z);
        }
    }

    /// A typed-result built-in under an axis subscript (`'+ r_/_12`): the
    /// whole subscripted function is wrapped, so the axis rule still
    /// sees the built-in itself.
    fn subscripted(&mut self, e: &mut Expr) {
        if let Kind::Axes { f, arity, .. } = &mut e.kind {
            *arity = self.dicts.axes.get(&e.id).copied().or(*arity);
            if let Some((t, n)) = self.dicts.prims.get(&f.id) {
                *e = typed_result(e, t, *n, &self.scope);
            }
        }
    }

    fn children(&mut self, e: &mut Expr) {
        let kids: Vec<&mut Expr> = match &mut e.kind {
            Kind::Array(items) => items.iter_mut().collect(),
            Kind::Axes { f: x, arity, .. } => {
                *arity = self.dicts.axes.get(&e.id).copied().or(*arity);
                vec![x]
            }
            Kind::Lam { body: x, .. } => vec![x],
            Kind::App(f, x) => vec![f, x],
            Kind::App2 { f, left, right } => vec![f, left, right],
            Kind::Set { value, body, .. } => vec![value, body],
            Kind::If { cond, then, other } => vec![cond, then, other],
            _ => Vec::new(),
        };
        for kid in kids {
            self.expr(kid);
        }
    }
}
