//! Class constraints on type variables (T5, T8), from one table: a
//! class is a name, the base types it admits and the phrase for errors.

use crate::ty::Type;

struct Class {
    name: &'static str,
    admits: fn(&Type) -> bool,
    phrase: &'static str,
}

const TABLE: [Class; 4] = [
    Class {
        name: "Num",
        admits: |t| matches!(t, Type::Int | Type::Float),
        phrase: "a number",
    },
    Class {
        name: "Truthy",
        admits: |t| matches!(t, Type::Bool | Type::Int),
        phrase: "Bool or Int",
    },
    Class {
        name: "Eq",
        admits: |t| {
            matches!(
                t,
                Type::Int | Type::Float | Type::Bool | Type::Char | Type::Box(_) | Type::Named(_)
            )
        },
        phrase: "Int, Float, Bool or Char",
    },
    Class {
        name: "Ord",
        admits: |t| matches!(t, Type::Int | Type::Float | Type::Char),
        phrase: "a number or Char",
    },
];

/// The base types a class may admit.
const BASE: [Type; 4] = [Type::Int, Type::Float, Type::Bool, Type::Char];

/// The classes a type variable must belong to (a set, one bit per row
/// of the table).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Classes(u8);

impl Classes {
    /// The class spelled `name` (`Num`, `Truthy`, `Eq`, `Ord`).
    pub fn named(name: &str) -> Option<Classes> {
        TABLE
            .iter()
            .position(|c| c.name == name)
            .map(|i| Classes(1 << i))
    }

    pub fn union(self, other: Classes) -> Classes {
        Classes(self.0 | other.0)
    }

    fn rows(self) -> impl Iterator<Item = &'static Class> {
        TABLE
            .iter()
            .enumerate()
            .filter(move |(i, _)| self.0 & (1 << i) != 0)
            .map(|(_, c)| c)
    }

    /// The names of its classes, in table order, leaving out a class
    /// another one implies (every number is ordered: `Num a` implies
    /// `Ord a` and `Eq a`).
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        let implied = move |c: &Class| {
            self.rows()
                .any(|d| d.name != c.name && BASE.iter().all(|t| !(d.admits)(t) || (c.admits)(t)))
        };
        self.rows().filter(move |c| !implied(c)).map(|c| c.name)
    }

    pub fn admits(self, t: &Type) -> bool {
        self.rows().all(|c| (c.admits)(t))
    }

    /// What a member looks like, for a mismatch message.
    pub fn describe(self) -> String {
        let fits: Vec<Type> = BASE.into_iter().filter(|t| self.admits(t)).collect();
        match (self.rows().count(), &fits[..]) {
            (1, _) => self.rows().map(|c| c.phrase).collect(),
            (_, [Type::Int, Type::Float]) => "a number".into(),
            _ => fits
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" or "),
        }
    }
}
