//! Expressions and function positions: currying (F1, F3), dyadic
//! application (E4), quotes and operands (F4, F8, F9), applied values
//! (F5), names (N5, L7) and axes (A6).

use xetal_base::{Diagnostic, Span};
use xetal_lex::{FuncName, Side};
use xetal_syntax::{Expr as Surface, ExprKind, Fun, FunKind};

use crate::lower::{Lower, err};
use xetal_ir::{Expr, Kind};

impl Lower {
    pub(crate) fn expr(&mut self, e: &Surface) -> Result<Expr, Diagnostic> {
        let kind = match &e.kind {
            ExprKind::Num(n) => Kind::Lit(*n),
            ExprKind::Strand(items) => {
                let items = items
                    .iter()
                    .map(|x| self.strand_item(x))
                    .collect::<Result<_, _>>()?;
                Kind::Array(items)
            }
            ExprKind::Var(v) => {
                let name = format!("{}{}", v.name, if v.mutable { "!" } else { "" });
                match &v.ns {
                    Some(ns) => Kind::Global(format!("{ns}:{name}")),
                    None => Kind::Var(name),
                }
            }
            ExprKind::Arg(side) => return self.arg(*side, e.span),
            ExprKind::Str(s) => Kind::Str(s.clone()),
            ExprKind::Unit => Kind::Unit,
            ExprKind::Pow { base, exp } => {
                let f = self.node(e.span, Kind::Prim("^".into()));
                let left = self.expr(base)?;
                // D-4: a negative literal exponent gives a Float result.
                let exp = match exp {
                    xetal_lex::Number::Int(i) if *i < 0 => xetal_lex::Number::Float(*i as f64),
                    other => *other,
                };
                let right = self.node(e.span, Kind::Lit(exp));
                Kind::App2 {
                    f: Box::new(f),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            ExprKind::Quote(f) | ExprKind::Fn(f) => return self.fun(f),
            ExprKind::Monadic { f, arg } => {
                let arg = self.expr(arg)?;
                return self.apply1(f, arg);
            }
            ExprKind::Dyadic { left, f, right } => {
                let (left, right) = (self.expr(left)?, self.expr(right)?);
                return self.dyadic(f, left, right, e.span);
            }
        };
        Ok(self.node(e.span, kind))
    }

    /// A function in value position (a train there is monadic, TR4).
    pub(crate) fn fun(&mut self, f: &Fun) -> Result<Expr, Diagnostic> {
        let kind = match &f.kind {
            FunKind::Name(n) => return Ok(self.name(n, f.span)),
            FunKind::Sym(s) => Kind::Prim(s.text().into()),
            FunKind::Arg(side) => return self.arg(*side, f.span),
            FunKind::Apply(e) => return self.expr(e),
            FunKind::Operand { operand, f: target } => {
                let target = self.fun(target)?;
                let operand = self.fun(operand)?;
                Kind::App(Box::new(target), Box::new(operand))
            }
            // D-7: `f_^n` is `n 'f_ p_ower`, waiting for its argument.
            FunKind::Power { f: base, count } => {
                let power = self.node(f.span, Kind::Prim("p_ower".into()));
                let base = self.fun(base)?;
                let count = self.node(f.span, Kind::Lit(xetal_lex::Number::Int(*count)));
                let partial = self.node(f.span, Kind::App(Box::new(power), Box::new(base)));
                Kind::App(Box::new(partial), Box::new(count))
            }
            FunKind::Lambda(l) => return self.lambda(l, f.span),
            FunKind::Train(fs) => return self.train_value(fs, f.span),
        };
        Ok(self.node(f.span, kind))
    }

    /// `f x`; a train is expanded in place.
    pub(crate) fn apply1(&mut self, f: &Fun, arg: Expr) -> Result<Expr, Diagnostic> {
        let span = f.span.join(arg.span);
        let f = self.fun(f)?;
        Ok(self.node(span, Kind::App(Box::new(f), Box::new(arg))))
    }

    /// `x f y`: the arguments are already lowered; `left` and `right` are
    /// only referenced once each.
    pub(crate) fn apply2(&mut self, f: &Fun, left: Expr, right: Expr) -> Result<Expr, Diagnostic> {
        if let FunKind::Train(fs) = &f.kind {
            return self.train_apply(fs, Some(left), right);
        }
        let span = left.span.join(right.span);
        let f = self.fun(f)?;
        Ok(self.node(
            span,
            Kind::App2 {
                f: Box::new(f),
                left: Box::new(left),
                right: Box::new(right),
            },
        ))
    }

    /// A name: a global when qualified, a local when bound by a lambda
    /// or local binding, otherwise a built-in; axes wrap it (A6).
    fn name(&mut self, n: &FuncName, span: Span) -> Expr {
        let spelled = n.spelled();
        let kind = match n.ns.as_deref() {
            Some(xetal_lex::SYSTEM) => Kind::Prim(format!("{}{spelled}", xetal_lex::SYSTEM)),
            Some(ns) => Kind::Global(format!("{ns}:{spelled}")),
            None if self.is_bound(&spelled) => Kind::Var(spelled),
            None => Kind::Prim(spelled),
        };
        let base = self.node(span, kind);
        if n.axes.is_empty() {
            return base;
        }
        self.node(
            span,
            Kind::Axes {
                axes: n.axes.clone(),
                arity: None,
                f: Box::new(base),
            },
        )
    }

    fn arg(&mut self, side: Side, span: Span) -> Result<Expr, Diagnostic> {
        if self.lambdas == 0 {
            return Err(err(
                "arg-outside-lambda",
                span,
                "_l and _r exist only inside a lambda",
            ));
        }
        let name = if side == Side::Left { "_l" } else { "_r" };
        Ok(self.node(span, Kind::Var(name.into())))
    }
}
