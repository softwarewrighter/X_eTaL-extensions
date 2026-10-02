//! Expressions: a left-to-right scan into value and function items,
//! operand binding (F8, F9), then a right-to-left reduction (F1, F3).

use xetal_base::{Diagnostic, Span};
use xetal_lex::TokenKind;

use crate::parser::{Parser, err};
use xetal_ast::{Expr, ExprKind, Fun, FunKind, check_depth};

pub(crate) enum Item {
    Value(Expr),
    Fun(Fun),
    /// A quoted function: an operand if a function follows, else a value.
    Quoted(Fun, Span),
}

impl Parser {
    /// True when the next token ends an expression.
    pub(crate) fn expr_is_empty(&mut self) -> bool {
        matches!(
            self.peek().map(|t| &t.kind),
            None | Some(
                TokenKind::Newline
                    | TokenKind::Semi
                    | TokenKind::Guard
                    | TokenKind::RParen
                    | TokenKind::RBracket
                    | TokenKind::RBrace
            )
        )
    }

    pub(crate) fn expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.here();
        let items = self.items()?;
        reduce(bind_operands(items), start)
    }

    /// Items up to the end of the expression.
    pub(crate) fn items(&mut self) -> Result<Vec<Item>, Diagnostic> {
        let mut items = Vec::new();
        while !self.expr_is_empty() {
            let token = self.next().expect("not empty");
            let item = self.item(token)?;
            if let Item::Value(Expr { depth, span, .. }) | Item::Fun(Fun { depth, span, .. }) =
                &item
            {
                check_depth(*depth, *span)?;
            }
            items.push(item);
        }
        Ok(items)
    }
}

/// A quoted function directly left of a function becomes its operand;
/// the operand nearest the function binds first (F8, F9).
pub(crate) fn bind_operands(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::new();
    for item in items {
        match item {
            Item::Fun(mut f) => {
                while let Some(Item::Quoted(..)) = out.last() {
                    let Some(Item::Quoted(operand, span)) = out.pop() else {
                        unreachable!()
                    };
                    let span = span.join(f.span);
                    let kind = FunKind::Operand {
                        operand: Box::new(operand),
                        f: Box::new(f),
                    };
                    f = Fun::new(kind, span);
                }
                out.push(Item::Fun(f));
            }
            other => out.push(other),
        }
    }
    out
}

/// Right to left: a function takes everything to its right; the single
/// value immediately to its left, if any, is its left argument.
fn reduce(items: Vec<Item>, start: Span) -> Result<Expr, Diagnostic> {
    let mut items: Vec<Item> = items
        .into_iter()
        .map(|item| match item {
            Item::Quoted(f, span) => Item::Value(Expr::new(ExprKind::Quote(Box::new(f)), span)),
            other => other,
        })
        .collect();
    let mut value = match rightmost(&mut items, start)? {
        Ok(value) => value,
        Err(function) => return Ok(function),
    };
    while let Some(item) = items.pop() {
        let f = match item {
            Item::Fun(f) => f,
            Item::Value(v) => {
                return Err(err(
                    "adjacent-values",
                    v.span,
                    "two values side by side (only literals of one kind form strands: numbers, or strings)",
                ));
            }
            Item::Quoted(..) => unreachable!("quotes were turned into values"),
        };
        value = apply(&mut items, f, value)?;
        check_depth(value.depth, value.span)?;
    }
    Ok(value)
}

/// The rightmost item: a value to start from (`Ok`), or a lone function
/// that is the whole expression (`Err`).
fn rightmost(items: &mut Vec<Item>, start: Span) -> Result<Result<Expr, Expr>, Diagnostic> {
    match items.pop() {
        None => Err(err("missing-value", start, "expected an expression")),
        Some(Item::Fun(f)) if items.is_empty() => {
            let span = f.span;
            Ok(Err(Expr::new(ExprKind::Fn(Box::new(f)), span)))
        }
        Some(Item::Fun(f)) => Err(err(
            "missing-argument",
            f.span,
            "this function needs an argument on its right (to pass it as a value, quote it)",
        )),
        Some(Item::Value(v)) => Ok(Ok(v)),
        Some(Item::Quoted(..)) => unreachable!("quotes were turned into values"),
    }
}

/// Apply `f` to `right`, dyadically when a value sits directly left.
fn apply(items: &mut Vec<Item>, f: Fun, right: Expr) -> Result<Expr, Diagnostic> {
    if let Some(Item::Value(_)) = items.last() {
        let Some(Item::Value(left)) = items.pop() else {
            unreachable!()
        };
        let span = left.span.join(right.span);
        let kind = ExprKind::Dyadic {
            left: Box::new(left),
            f: Box::new(f),
            right: Box::new(right),
        };
        return Ok(Expr::new(kind, span));
    }
    if matches!(f.kind, FunKind::Sym(_)) {
        return Err(err(
            "symbol-needs-left",
            f.span,
            "a symbol function needs a left argument (to negate use n_eg; for a partial application use a lambda)",
        ));
    }
    let span = f.span.join(right.span);
    let kind = ExprKind::Monadic {
        f: Box::new(f),
        arg: Box::new(right),
    };
    Ok(Expr::new(kind, span))
}
