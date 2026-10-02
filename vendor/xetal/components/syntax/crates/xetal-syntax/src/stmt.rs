//! Programs and statements: bindings (S1), guards (G1), separators
//! (S2, S3).

use xetal_base::Diagnostic;
use xetal_lex::{Token, TokenKind};

use crate::parser::{Parser, err};
use xetal_ast::{Program, Stmt, Target};

impl Parser {
    pub(crate) fn program(&mut self) -> Result<Program, Diagnostic> {
        let stmts = self.statements(false)?;
        if let Some(t) = self.peek() {
            let span = t.span;
            return Err(err("unexpected-token", span, "unexpected token"));
        }
        Ok(Program { stmts })
    }

    /// Statements separated by newlines or `;`, up to the end or `}`.
    pub(crate) fn statements(&mut self, in_lambda: bool) -> Result<Vec<Stmt>, Diagnostic> {
        let mut stmts = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Newline | TokenKind::Semi) => {
                    self.pos += 1;
                }
                None | Some(TokenKind::RBrace) => return Ok(stmts),
                Some(_) => {
                    stmts.push(self.statement(in_lambda)?);
                    match self.peek().map(|t| &t.kind) {
                        None | Some(TokenKind::Newline | TokenKind::Semi | TokenKind::RBrace) => {}
                        Some(_) => {
                            let span = self.here();
                            return Err(err(
                                "unexpected-token",
                                span,
                                "expected the end of the statement",
                            ));
                        }
                    }
                }
            }
        }
    }

    fn statement(&mut self, in_lambda: bool) -> Result<Stmt, Diagnostic> {
        let is_binding = self
            .tokens
            .get(self.pos + 1)
            .is_some_and(|t| t.kind == TokenKind::Assign)
            && self.tokens.get(self.pos).and_then(target).is_some();
        if is_binding {
            let name = self.next().expect("checked");
            let assign = self.next().expect("checked");
            let target = target(&name).expect("checked");
            if self.expr_is_empty() {
                return Err(err(
                    "missing-value",
                    assign.span,
                    "a binding needs a value after `:=`",
                ));
            }
            let value = self.expr()?;
            let span = name.span.join(value.span);
            return Ok(Stmt::Bind {
                target,
                value,
                span,
            });
        }
        let cond = self.expr()?;
        if self.peek().is_some_and(|t| t.kind == TokenKind::Guard) {
            let guard = self.next().expect("checked");
            if !in_lambda {
                return Err(err(
                    "guard-outside-lambda",
                    guard.span,
                    "a guard `?` belongs inside a lambda",
                ));
            }
            let result = self.expr()?;
            let span = cond.span.join(result.span);
            return Ok(Stmt::Guard { cond, result, span });
        }
        Ok(Stmt::Expr(cond))
    }
}

/// A bindable name.
pub(crate) fn target(token: &Token) -> Option<Target> {
    match &token.kind {
        TokenKind::Var(v) => Some(Target::Var(v.clone())),
        TokenKind::Func(f) => Some(Target::Func(f.clone())),
        _ => None,
    }
}
