//! Item-by-item built-ins call their operand once per item, in order.

use std::rc::Rc;

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_map::{each, table};
use xetal_value::{Caller, Prim, Value};

/// Records every argument and answers with a partial application (for
/// the first of two arguments) or the argument itself.
#[derive(Default)]
struct Log(Vec<String>);

impl<'a> Caller<'a> for Log {
    fn call(&mut self, f: &Value<'a>, x: Value<'a>, _: Span) -> Result<Value<'a>, Diagnostic> {
        self.0.push(x.to_string());
        Ok(match f {
            Value::Prim(p) if p.args.is_empty() && p.arity == 2 => partial(vec![x]),
            _ => x,
        })
    }
}

fn partial<'a>(args: Vec<Value<'a>>) -> Value<'a> {
    Value::Prim(Rc::new(Prim {
        name: "f",
        arity: 2,
        args,
    }))
}

fn vector(items: &[i64]) -> Value<'static> {
    let data = items.iter().map(|i| Value::Int(*i)).collect();
    Value::Array(Rc::new(Array::vector(data)))
}

#[test]
fn each_visits_items_in_order() {
    let mut log = Log::default();
    let id = Value::Prim(Rc::new(Prim {
        name: "g",
        arity: 1,
        args: Vec::new(),
    }));
    let out = each(&id, &vector(&[3, 1, 2]), Span::default(), &mut log).unwrap();
    assert_eq!(out.to_string(), "3 1 2");
    assert_eq!(log.0, ["3", "1", "2"]);
}

#[test]
fn table_fixes_each_left_item_once() {
    let mut log = Log::default();
    let f = partial(Vec::new());
    let out = table(
        &f,
        &vector(&[1, 2]),
        &vector(&[7, 8]),
        Span::default(),
        &mut log,
    )
    .unwrap();
    assert_eq!(out.to_string(), "7 8\n7 8");
    assert_eq!(log.0, ["1", "7", "8", "2", "7", "8"]);
}
