//! A descriptor after validation: owned text, and function pointers
//! that are valid while the library that defines them stays loaded.

use crate::model::InvokeFnV1;

#[derive(Clone, Debug)]
pub struct ValidatedFunction {
    name: String,
    arity: u32,
    signature: String,
    doc: String,
    invoke: InvokeFnV1,
}

impl PartialEq for ValidatedFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.arity == other.arity
            && self.signature == other.signature
            && self.doc == other.doc
    }
}

impl Eq for ValidatedFunction {}

impl ValidatedFunction {
    pub(crate) const fn new(
        name: String,
        arity: u32,
        signature: String,
        doc: String,
        invoke: InvokeFnV1,
    ) -> Self {
        Self {
            name,
            arity,
            signature,
            doc,
            invoke,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn arity(&self) -> u32 {
        self.arity
    }

    /// The function's X_eTaL type, for example `Char -> Char`.
    #[must_use]
    pub fn signature(&self) -> &str {
        &self.signature
    }

    #[must_use]
    pub fn doc(&self) -> &str {
        &self.doc
    }

    /// The trampoline; only valid while its library is loaded.
    #[must_use]
    pub const fn invoke(&self) -> InvokeFnV1 {
        self.invoke
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedExtension {
    name: String,
    version: String,
    functions: Vec<ValidatedFunction>,
}

impl ValidatedExtension {
    pub(crate) const fn new(
        name: String,
        version: String,
        functions: Vec<ValidatedFunction>,
    ) -> Self {
        Self {
            name,
            version,
            functions,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[must_use]
    pub fn functions(&self) -> &[ValidatedFunction] {
        &self.functions
    }

    #[must_use]
    pub fn function(&self, name: &str) -> Option<&ValidatedFunction> {
        self.functions.iter().find(|f| f.name == name)
    }
}
