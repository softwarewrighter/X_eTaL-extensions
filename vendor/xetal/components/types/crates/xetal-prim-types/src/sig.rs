//! Reading a signature such as `(Num a, Truthy b) => a -> a -> b` into
//! a type with fresh, constrained variables.

use std::collections::HashMap;

use xetal_base::Diagnostic;
use xetal_ty::{Classes, Type, Unifier};

/// The signature's type, with a fresh variable for each name.
pub fn read(sig: &str, u: &mut Unifier) -> Result<Type, Diagnostic> {
    let (context, ty) = sig.rsplit_once("=>").unwrap_or(("", sig));
    let mut vars = HashMap::new();
    for part in context.split(',') {
        let words: Vec<&str> = part
            .split(|c: char| c == '(' || c == ')' || c.is_whitespace())
            .filter(|w| !w.is_empty())
            .collect();
        if let [class, var] = words[..] {
            let class = Classes::named(class).ok_or_else(|| bad(sig))?;
            vars.insert(var.to_string(), u.fresh_in(class));
        }
    }
    let spaced = ty.replace('(', " ( ").replace(')', " ) ");
    let mut tokens: Vec<&str> = spaced.split_whitespace().collect();
    tokens.reverse();
    let t = arrow(&mut tokens, &mut vars, u)?;
    match tokens.is_empty() {
        true => Ok(t),
        false => Err(bad(sig)),
    }
}

fn bad(sig: &str) -> Diagnostic {
    Diagnostic::new("internal", format!("bad built-in signature `{sig}`"))
}

type Vars = HashMap<String, Type>;

/// `atom (-> arrow)?`, right associative.
fn arrow(tokens: &mut Vec<&str>, vars: &mut Vars, u: &mut Unifier) -> Result<Type, Diagnostic> {
    let a = atom(tokens, vars, u)?;
    if tokens.last() != Some(&"->") {
        return Ok(a);
    }
    tokens.pop();
    Ok(Type::Fn(Box::new(a), Box::new(arrow(tokens, vars, u)?)))
}

fn atom(tokens: &mut Vec<&str>, vars: &mut Vars, u: &mut Unifier) -> Result<Type, Diagnostic> {
    let token = tokens.pop().ok_or_else(|| bad("(ends early)"))?;
    Ok(match token {
        "Int" => Type::Int,
        "Float" => Type::Float,
        "Bool" => Type::Bool,
        "Char" => Type::Char,
        "Unit" => Type::Unit,
        "Box" => Type::Box(Box::new(atom(tokens, vars, u)?)),
        "(" => {
            let t = arrow(tokens, vars, u)?;
            (tokens.pop() == Some(")"))
                .then_some(t)
                .ok_or_else(|| bad("(unclosed)"))?
        }
        name => vars
            .entry(name.to_string())
            .or_insert_with(|| u.fresh())
            .clone(),
    })
}
