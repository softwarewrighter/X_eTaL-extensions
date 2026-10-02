//! What the type checker tells the elaborator, keyed by Core node.

use std::collections::HashMap;

use xetal_base::NodeId;
use xetal_ty::{Type, TypeVar};

/// Number-type facts from inference, with every type resolved.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Dicts {
    /// A generalized binding's value node and its quantified `Num`
    /// variables: one hidden parameter each, in this order.
    pub params: HashMap<NodeId, Vec<TypeVar>>,
    /// A use of such a binding (a `Var` or `Global` node) and the number
    /// types it passes, in the order of the binding's `params`.
    pub args: HashMap<NodeId, Vec<Type>>,
    /// Integer literals and their types.
    pub lits: HashMap<NodeId, Type>,
    /// Built-ins that make a result with no typed item (reduce's
    /// identity), with their result types and arities.
    pub prims: HashMap<NodeId, (Type, usize)>,
    /// Functions under an axis subscript (A6), and how many arguments
    /// their types say they take.
    pub axes: HashMap<NodeId, usize>,
}
