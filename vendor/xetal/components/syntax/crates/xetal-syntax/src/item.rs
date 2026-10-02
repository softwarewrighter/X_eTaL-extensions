//! Expression items: values (numbers and strands, variables, strings,
//! Unit, parentheses, quotes) and functions, with literal exponents.

use xetal_base::{Diagnostic, Span};
use xetal_lex::{Number, Token, TokenKind};

use crate::expr::Item;
use crate::parser::{Parser, err};
use xetal_ast::{Expr, ExprKind, Fun, FunKind};

impl Parser {
    pub(crate) fn item(&mut self, token: Token) -> Result<Item, Diagnostic> {
        let noun = |kind| Expr::new(kind, token.span);
        let value = match token.kind {
            TokenKind::Num(n) => {
                let mut parts = vec![self.exponent(noun(ExprKind::Num(n)))?];
                while let Some(Token {
                    kind: TokenKind::Num(m),
                    span,
                }) = self.peek().cloned()
                {
                    self.pos += 1;
                    parts.push(self.exponent(Expr::new(ExprKind::Num(m), span))?);
                }
                if parts.len() == 1 {
                    parts.remove(0)
                } else {
                    let span = parts[0].span.join(parts[parts.len() - 1].span);
                    Expr::new(ExprKind::Strand(parts), span)
                }
            }
            TokenKind::Var(v) => self.exponent(noun(ExprKind::Var(v)))?,
            TokenKind::LamArg {
                side,
                applied: false,
            } => self.exponent(noun(ExprKind::Arg(side)))?,
            TokenKind::Str(s) => self.strings(noun(ExprKind::Str(s))),
            TokenKind::Unit => noun(ExprKind::Unit),
            TokenKind::LParen => return self.paren(token.span),
            TokenKind::Quote => return self.quote(token.span),
            _ => return self.function(token).map(Item::Fun),
        };
        Ok(Item::Value(value))
    }

    /// A string literal and any that follow it: a strand of strings is
    /// a nested vector (B14).
    fn strings(&mut self, first: Expr) -> Expr {
        let mut parts = vec![first];
        while let Some(Token {
            kind: TokenKind::Str(s),
            span,
        }) = self.peek().cloned()
        {
            self.pos += 1;
            parts.push(Expr::new(ExprKind::Str(s), span));
        }
        match parts.len() {
            1 => parts.remove(0),
            _ => {
                let span = parts[0].span.join(parts[parts.len() - 1].span);
                Expr::new(ExprKind::Strand(parts), span)
            }
        }
    }

    /// A function token: a name, a symbol, `_l_`, a lambda or a train.
    fn function(&mut self, token: Token) -> Result<Fun, Diagnostic> {
        let kind = match token.kind {
            TokenKind::Func(name) => match self.peek().map(|t| (t.kind.clone(), t.span)) {
                // D-7: `f_^3`, a power (the lexer allows whole counts only).
                Some((TokenKind::Exp(Number::Int(count)), span)) => {
                    self.pos += 1;
                    let f = Box::new(Fun::new(FunKind::Name(name), token.span));
                    let kind = FunKind::Power { f, count };
                    return Ok(Fun::new(kind, token.span.join(span)));
                }
                _ => FunKind::Name(name),
            },
            TokenKind::Sym(sym) => FunKind::Sym(sym),
            TokenKind::LamArg {
                side,
                applied: true,
            } => FunKind::Arg(side),
            TokenKind::LBrace => return self.lambda(token.span),
            TokenKind::LBracket => return self.train(token.span),
            other => {
                let message = match other {
                    TokenKind::Assign => {
                        "`:=` binds only at the start of a statement (top level or in braces)"
                    }
                    TokenKind::Arrow => "`->` separates parameters from the body inside `{ }`",
                    TokenKind::Lazy => "`~` marks a lazy parameter inside `{ ... -> }`",
                    TokenKind::Apply => "`_` after `)` applies a parenthesized function value",
                    TokenKind::Exp(_) => "an exponent must touch a value",
                    _ => "unexpected token",
                };
                return Err(err("unexpected-token", token.span, message));
            }
        };
        Ok(Fun::new(kind, token.span))
    }

    /// A literal exponent touching the value just parsed (D-2, D-3).
    fn exponent(&mut self, base: Expr) -> Result<Expr, Diagnostic> {
        match self.peek().cloned() {
            Some(Token {
                kind: TokenKind::Exp(exp),
                span,
            }) => {
                self.pos += 1;
                let span = base.span.join(span);
                let kind = ExprKind::Pow {
                    base: Box::new(base),
                    exp,
                };
                Ok(Expr::new(kind, span))
            }
            _ => Ok(base),
        }
    }

    /// `( expr )`: a value, a function, or with `)_` an applied value.
    fn paren(&mut self, open: Span) -> Result<Item, Diagnostic> {
        self.enter(open)?;
        let item = self.paren_inner(open);
        self.leave();
        item
    }

    fn paren_inner(&mut self, open: Span) -> Result<Item, Diagnostic> {
        self.newline_is_space.push(true);
        let inner = if self.expr_is_empty() {
            Err(err(
                "missing-value",
                self.here(),
                "expected an expression inside `( )`",
            ))
        } else {
            self.expr()
        };
        let close = self.next();
        self.newline_is_space.pop();
        let inner = inner?;
        let span = match close {
            Some(Token {
                kind: TokenKind::RParen,
                span,
            }) => open.join(span),
            Some(t) => return Err(err("unexpected-token", t.span, "expected `)`")),
            None => return Err(err("unclosed", open, "this `(` is never closed")),
        };
        if self.peek().is_some_and(|t| t.kind == TokenKind::Apply) {
            let apply = self.next().expect("checked");
            let kind = FunKind::Apply(Box::new(inner));
            return Ok(Item::Fun(Fun::new(kind, span.join(apply.span))));
        }
        match inner.kind {
            ExprKind::Fn(f) => match self.peek() {
                Some(t) if matches!(t.kind, TokenKind::Exp(_)) => Err(err(
                    "bad-exponent",
                    t.span,
                    "a power goes on a function name (`f_^3`); otherwise use `p_ower`",
                )),
                _ => Ok(Item::Fun(Fun::new(f.kind, span))),
            },
            kind => Ok(Item::Value(self.exponent(Expr::new(kind, span))?)),
        }
    }

    /// `'f`: a function name, symbol, lambda or train as a value (F4).
    fn quote(&mut self, quote: Span) -> Result<Item, Diagnostic> {
        let token = self
            .next()
            .expect("the lexer requires a quote to touch its function");
        if matches!(token.kind, TokenKind::Var(_)) {
            let span = quote.join(token.span);
            return Err(err(
                "bad-quote",
                span,
                "only functions can be quoted; a variable is already a value",
            ));
        }
        let fun = self.function(token)?;
        let span = quote.join(fun.span);
        Ok(Item::Quoted(fun, span))
    }
}
