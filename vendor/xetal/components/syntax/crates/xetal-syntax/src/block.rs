//! Lambdas (L1-L7) and trains (TR1-TR3).

use xetal_base::{Diagnostic, Span};
use xetal_lex::TokenKind;

use crate::expr::{Item, bind_operands};
use crate::parser::{Parser, err};
use crate::stmt::target;
use xetal_ast::{Fun, FunKind, Lambda, Param, Params, stmt_args};

impl Parser {
    /// `{ [params ->] body }`.
    pub(crate) fn lambda(&mut self, open: Span) -> Result<Fun, Diagnostic> {
        self.enter(open)?;
        self.newline_is_space.push(false);
        let result = self.lambda_inner(open);
        self.newline_is_space.pop();
        self.leave();
        result
    }

    fn lambda_inner(&mut self, open: Span) -> Result<Fun, Diagnostic> {
        while self
            .tokens
            .get(self.pos)
            .is_some_and(|t| t.kind == TokenKind::Newline)
        {
            self.pos += 1;
        }
        let named = self.params()?;
        let body = self.statements(true)?;
        let span = match self.next() {
            Some(t) if t.kind == TokenKind::RBrace => open.join(t.span),
            Some(t) => return Err(err("unexpected-token", t.span, "expected `}`")),
            None => return Err(err("unclosed", open, "this `{` is never closed")),
        };
        let (left, right) = body.iter().fold((false, false), |acc, s| stmt_args(s, acc));
        let params = match (named, left, right) {
            (Some(_), true, _) | (Some(_), _, true) => {
                return Err(err(
                    "bad-lambda",
                    span,
                    "do not mix named parameters with _l / _r",
                ));
            }
            (Some(p), _, _) => p,
            (None, false, true) => Params::Right,
            (None, true, true) => Params::LeftRight,
            (None, true, false) => {
                return Err(err(
                    "bad-lambda",
                    span,
                    "a lambda using _l must also use _r; otherwise name the parameters",
                ));
            }
            (None, false, false) => {
                return Err(err(
                    "bad-lambda",
                    span,
                    "a lambda needs _r, _l and _r, named parameters, or `@ ->` for a niladic function",
                ));
            }
        };
        let kind = FunKind::Lambda(Lambda { params, body });
        Ok(Fun::new(kind, span))
    }

    /// Named parameters before `->`, if the lambda has them.
    fn params(&mut self) -> Result<Option<Params>, Diagnostic> {
        let mut i = self.pos;
        while self.tokens.get(i).is_some_and(|t| {
            matches!(
                t.kind,
                TokenKind::Lazy
                    | TokenKind::Var(_)
                    | TokenKind::Func(_)
                    | TokenKind::Unit
                    | TokenKind::Num(_)
            )
        }) {
            i += 1;
        }
        if self
            .tokens
            .get(i)
            .is_none_or(|t| t.kind != TokenKind::Arrow)
        {
            return Ok(None);
        }
        let tokens: Vec<_> = self.tokens[self.pos..i].to_vec();
        self.pos = i + 1;
        if tokens.len() == 1 && tokens[0].kind == TokenKind::Unit {
            return Ok(Some(Params::Niladic));
        }
        let mut params: Vec<Param> = Vec::new();
        let mut lazy = None;
        for t in tokens {
            match (&t.kind, target(&t)) {
                (TokenKind::Lazy, _) if lazy.is_none() => lazy = Some(t.span),
                (_, Some(name)) => {
                    if params.iter().any(|p| p.name == name) {
                        return Err(err("bad-lambda", t.span, "a parameter name appears twice"));
                    }
                    let span = lazy.map_or(t.span, |l| l.join(t.span));
                    params.push(Param {
                        name,
                        lazy: lazy.take().is_some(),
                        span,
                    });
                }
                (TokenKind::Unit, _) => {
                    return Err(err("bad-lambda", t.span, "`@` must be the only parameter"));
                }
                _ => return Err(err("bad-lambda", t.span, "a parameter must be a name")),
            }
        }
        Ok(Some(Params::Named(params)))
    }

    /// `[F G H]` fork, `[F G]` atop; longer trains group from the right.
    pub(crate) fn train(&mut self, open: Span) -> Result<Fun, Diagnostic> {
        self.enter(open)?;
        self.newline_is_space.push(true);
        let items = self.items();
        let close = self.next();
        self.newline_is_space.pop();
        self.leave();
        let items = bind_operands(items?);
        let span = match close {
            Some(t) if t.kind == TokenKind::RBracket => open.join(t.span),
            Some(t) => return Err(err("unexpected-token", t.span, "expected `]`")),
            None => return Err(err("unclosed", open, "this `[` is never closed")),
        };
        let mut funs = Vec::new();
        for item in items {
            match item {
                Item::Fun(f) => funs.push(f),
                Item::Value(v) => {
                    return Err(err("bad-train", v.span, "a train holds functions only"));
                }
                Item::Quoted(_, s) => {
                    return Err(err("bad-train", s, "a train holds functions only"));
                }
            }
        }
        if funs.len() < 2 {
            return Err(err(
                "bad-train",
                span,
                "a train needs at least two functions",
            ));
        }
        Ok(group(funs, span))
    }
}

fn group(mut funs: Vec<Fun>, span: Span) -> Fun {
    if funs.len() > 3 {
        let keep = if funs.len().is_multiple_of(2) { 1 } else { 2 };
        let rest: Vec<Fun> = funs.split_off(keep);
        let rest_span = rest[0].span.join(rest[rest.len() - 1].span);
        funs.push(group(rest, rest_span));
    }
    Fun::new(FunKind::Train(funs), span)
}
