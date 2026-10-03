//! The text protocol of the `ext:` channel (docs/bridge.md).
//!
//! A facade passes each argument with `[]N_PUT`, its kind and shape in
//! the path, and calls with `[]N_GET`:
//!
//! ```text
//! t []N_PUT "ext:hello/shout?text"                       # an argument: text
//! (f_ormat r_avel x) []N_PUT "ext:hello/sum?float=" c_at f_ormat s_hape x
//! []N_GET "ext:hello/sum"                                 # the call: its reply as text
//! []N_GET "ext:hello/sum?shape"                           # the reply's shape ("" for a scalar)
//! []N_GET "ext:hello/sum?kind"                            # int, float, bool, text or chars
//! ```

use xetal_ext_loader::{Array, ArrayData, Value};

/// What a path asks for, after `ext:`.
#[derive(Debug, PartialEq, Eq)]
pub enum Request<'p> {
    /// `EXT/FN` (get: call; put: an argument, as text).
    Call { ext: &'p str, function: &'p str },
    /// `EXT/FN?KIND[=SHAPE]` (put: an argument of that kind and shape).
    Argument {
        ext: &'p str,
        function: &'p str,
        kind: Kind,
        shape: Vec<usize>,
    },
    /// `EXT/FN?shape` (get: the last reply's shape).
    Shape { ext: &'p str, function: &'p str },
    /// `EXT/FN?kind` (get: the last reply's kind).
    KindOf { ext: &'p str, function: &'p str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Chars,
    Int,
    Float,
    Bool,
}

impl Kind {
    fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "text" => Self::Text,
            "chars" => Self::Chars,
            "int" => Self::Int,
            "float" => Self::Float,
            "bool" => Self::Bool,
            _ => return None,
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Chars => "chars",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
        }
    }
}

/// Reads `EXT/FN[?QUERY]`.
pub fn parse(path: &str) -> Result<Request<'_>, String> {
    let (target, query) = match path.split_once('?') {
        Some((t, q)) => (t, Some(q)),
        None => (path, None),
    };
    let (ext, function) = target
        .split_once('/')
        .filter(|(e, f)| !e.is_empty() && !f.is_empty() && !f.contains('/'))
        .ok_or("expected ext:EXTENSION/FUNCTION")?;
    let Some(query) = query else {
        return Ok(Request::Call { ext, function });
    };
    let (word, shape) = match query.split_once('=') {
        Some((w, s)) => (w, Some(s)),
        None => (query, None),
    };
    match (word, shape) {
        ("shape", None) => Ok(Request::Shape { ext, function }),
        ("kind", None) => Ok(Request::KindOf { ext, function }),
        (w, shape) => {
            let kind = Kind::parse(w).ok_or_else(|| {
                format!("unknown ?{w}: expected text, chars, int, float, bool, shape or kind")
            })?;
            let shape = match shape {
                None => Vec::new(),
                Some(s) => s
                    .split_whitespace()
                    .map(|n| n.parse::<usize>().map_err(|_| format!("bad shape {s:?}")))
                    .collect::<Result<_, _>>()?,
            };
            if kind == Kind::Text && !shape.is_empty() {
                return Err("?text takes no shape (use ?chars=SHAPE)".into());
            }
            Ok(Request::Argument {
                ext,
                function,
                kind,
                shape,
            })
        }
    }
}

/// The argument a put carries: its body read as `kind`, shaped.
pub fn argument(kind: Kind, shape: Vec<usize>, body: &str) -> Result<Value, String> {
    if kind == Kind::Text {
        return Ok(Value::Text(body.to_owned()));
    }
    let words = || body.split_whitespace();
    let data = match kind {
        Kind::Text => unreachable!(),
        Kind::Chars => ArrayData::Char(body.chars().filter(|c| *c != '\n').collect()),
        Kind::Int => ArrayData::Int(
            words()
                .map(|w| w.parse::<i64>().map_err(|_| format!("{w:?} is not an Int")))
                .collect::<Result<_, _>>()?,
        ),
        Kind::Float => ArrayData::Float(
            words()
                .map(|w| {
                    w.parse::<f64>()
                        .map_err(|_| format!("{w:?} is not a number"))
                })
                .collect::<Result<_, _>>()?,
        ),
        Kind::Bool => ArrayData::Bool(
            words()
                .map(|w| match w {
                    "1" => Ok(true),
                    "0" => Ok(false),
                    _ => Err(format!("{w:?} is not a Bool (1 or 0)")),
                })
                .collect::<Result<_, _>>()?,
        ),
    };
    if shape.is_empty() {
        return match data {
            ArrayData::Int(v) if v.len() == 1 => Ok(Value::Int(v[0])),
            ArrayData::Float(v) if v.len() == 1 => Ok(Value::Float(v[0])),
            ArrayData::Bool(v) if v.len() == 1 => Ok(Value::Bool(v[0])),
            ArrayData::Char(v) => Ok(Value::Text(v.into_iter().collect())),
            d => Err(format!("a scalar needs one value, given {}", d.len())),
        };
    }
    Array::new(shape, data)
        .map(Value::Array)
        .map_err(|e| e.to_string())
}

/// A reply as the text a facade reads back: text as it is; numbers
/// and Bools ravelled, separated by spaces (`n_umbers` reads them).
pub fn reply(value: &Value) -> String {
    fn join<T>(v: &[T], f: impl Fn(&T) -> String) -> String {
        v.iter().map(f).collect::<Vec<_>>().join(" ")
    }
    match value {
        Value::Bool(b) => u8::from(*b).to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(x) => float(*x),
        Value::Text(t) => t.clone(),
        Value::Array(a) => match a.data() {
            ArrayData::Bool(v) => join(v, |b| u8::from(*b).to_string()),
            ArrayData::Int(v) => join(v, ToString::to_string),
            ArrayData::Float(v) => join(v, |x| float(*x)),
            ArrayData::Char(v) => v.iter().collect(),
        },
    }
}

/// A Float as the shortest text that reads back as the same number
/// (`1.0`, `0.30000000000000004`, `1e-20`, `1e300`); `n_umbers` reads
/// every form.
fn float(x: f64) -> String {
    format!("{x:?}")
}

/// The reply's shape: `""` for a scalar or text, else the axes.
pub fn shape(value: &Value) -> String {
    match value {
        Value::Array(a) => a
            .shape()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
        Value::Text(t) => t.chars().count().to_string(),
        _ => String::new(),
    }
}

/// The reply's kind.
pub fn kind(value: &Value) -> Kind {
    match value {
        Value::Bool(_) => Kind::Bool,
        Value::Int(_) => Kind::Int,
        Value::Float(_) => Kind::Float,
        Value::Text(_) => Kind::Text,
        Value::Array(a) => match a.data() {
            ArrayData::Bool(_) => Kind::Bool,
            ArrayData::Int(_) => Kind::Int,
            ArrayData::Float(_) => Kind::Float,
            ArrayData::Char(_) => Kind::Chars,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        assert_eq!(
            parse("hello/shout"),
            Ok(Request::Call {
                ext: "hello",
                function: "shout"
            })
        );
        assert_eq!(
            parse("hello/sum?float=2 3"),
            Ok(Request::Argument {
                ext: "hello",
                function: "sum",
                kind: Kind::Float,
                shape: vec![2, 3]
            })
        );
        assert_eq!(
            parse("hello/sum?int"),
            Ok(Request::Argument {
                ext: "hello",
                function: "sum",
                kind: Kind::Int,
                shape: vec![]
            })
        );
        assert_eq!(
            parse("hello/sum?int="),
            Ok(Request::Argument {
                ext: "hello",
                function: "sum",
                kind: Kind::Int,
                shape: vec![]
            })
        );
        assert_eq!(
            parse("hello/sum?shape"),
            Ok(Request::Shape {
                ext: "hello",
                function: "sum"
            })
        );
        assert_eq!(
            parse("hello/sum?kind"),
            Ok(Request::KindOf {
                ext: "hello",
                function: "sum"
            })
        );
        assert!(parse("hello").is_err());
        assert!(parse("hello/a/b").is_err());
        assert!(parse("/x").is_err());
        assert!(
            parse("hello/sum?wat")
                .unwrap_err()
                .starts_with("unknown ?wat")
        );
        assert!(
            parse("hello/sum?float=2 x")
                .unwrap_err()
                .starts_with("bad shape")
        );
        assert!(parse("hello/sum?text=3").is_err());
    }

    #[test]
    fn arguments() {
        assert_eq!(
            argument(Kind::Text, vec![], "a b\nc"),
            Ok(Value::Text("a b\nc".into()))
        );
        assert_eq!(argument(Kind::Int, vec![], "42"), Ok(Value::Int(42)));
        assert_eq!(
            argument(Kind::Float, vec![], "-2.5"),
            Ok(Value::Float(-2.5))
        );
        assert_eq!(argument(Kind::Bool, vec![], "1"), Ok(Value::Bool(true)));
        assert_eq!(
            argument(Kind::Float, vec![2, 2], "1 2.5\n3 0.00000000000000000001"),
            Ok(Value::Array(
                Array::new(vec![2, 2], ArrayData::Float(vec![1.0, 2.5, 3.0, 1e-20])).unwrap()
            ))
        );
        assert_eq!(
            argument(Kind::Chars, vec![2, 2], "ab\ncd"),
            Ok(Value::Array(
                Array::new(vec![2, 2], ArrayData::Char("abcd".chars().collect())).unwrap()
            ))
        );
        assert!(argument(Kind::Int, vec![], "1 2").is_err());
        assert!(argument(Kind::Int, vec![], "1.5").is_err());
        assert!(argument(Kind::Bool, vec![1], "2").is_err());
        assert_eq!(
            argument(Kind::Int, vec![2, 2], "1 2 3"),
            Err("shape says 4 elements, data has 3".into())
        );
    }

    #[test]
    fn replies() {
        let m = Value::Array(
            Array::new(
                vec![2, 3],
                ArrayData::Float(vec![0.1 + 0.2, 1.0, -2.5, 1e-20, 1e300, 7.0]),
            )
            .unwrap(),
        );
        assert_eq!(reply(&m), "0.30000000000000004 1.0 -2.5 1e-20 1e300 7.0");
        assert_eq!(shape(&m), "2 3");
        assert_eq!(kind(&m), Kind::Float);
        assert_eq!(reply(&Value::Int(42)), "42");
        assert_eq!(shape(&Value::Int(42)), "");
        assert_eq!(reply(&Value::Bool(true)), "1");
        assert_eq!(reply(&Value::Text("CAF\u{c9}".into())), "CAF\u{c9}");
        assert_eq!(shape(&Value::Text("CAF\u{c9}".into())), "4");
        assert_eq!(kind(&Value::Text("x".into())), Kind::Text);
    }

    #[test]
    fn numbers_round_trip() {
        for x in [
            0.1 + 0.2,
            1e-20,
            -3.5e-300,
            123_456_789.123_456_78,
            f64::MIN_POSITIVE,
        ] {
            let back: f64 = reply(&Value::Float(x)).parse().unwrap();
            assert_eq!(back.to_bits(), x.to_bits());
        }
    }
}
