//! The hello extension loaded both ways -- its shared library from its
//! package, and linked statically -- passing the same tests.

use std::fs;
use std::path::PathBuf;

use xetal_ext_loader::{
    Array, ArrayData, CallError, LoadError, Package, Provider, Registry, Value,
};

/// Where Cargo put hello's cdylib: beside this test binary.
fn build_dirs() -> Vec<PathBuf> {
    let exe = std::env::current_exe().unwrap();
    vec![exe.parent().unwrap().to_owned()]
}

fn hello_package() -> Package {
    Package::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../extensions/hello"
    ))
    .unwrap()
}

fn dynamic() -> Registry {
    let mut r = Registry::new();
    assert_eq!(
        r.load_package(&hello_package(), &build_dirs()).unwrap(),
        "hello"
    );
    r
}

fn statik() -> Registry {
    let mut r = Registry::new();
    r.load_static(xetal_ext_hello::__xetal_extension::descriptor)
        .unwrap();
    r
}

fn both() -> [Registry; 2] {
    [dynamic(), statik()]
}

#[test]
fn providers() {
    let d = dynamic();
    match d.provider("hello") {
        Some(Provider::Dynamic(p)) => assert!(p.ends_with(hello_package().library_file())),
        other => panic!("{other:?}"),
    }
    assert_eq!(statik().provider("hello"), Some(&Provider::Static));
}

#[test]
fn package_manifest() {
    let p = hello_package();
    let m = p.manifest();
    assert_eq!(
        (m.name.as_str(), m.version.as_str(), m.abi),
        ("hello", "0.1.0", 1)
    );
    assert!(p.facade().ends_with("Hello.xtl"));
}

#[test]
fn calls() {
    for r in both() {
        assert_eq!(r.extensions(), ["hello"]);
        assert_eq!(r.version("hello"), Some("0.1.0"));
        assert_eq!(r.call("hello", "answer", &[]), Ok(Value::Int(42)));
        assert_eq!(
            r.call("hello", "add", &[Value::Int(40), Value::Int(2)]),
            Ok(Value::Int(42))
        );
        assert_eq!(
            r.call("hello", "shout", &[Value::Text("x_etal".into())]),
            Ok(Value::Text("X_ETAL".into()))
        );
        let m = Value::Array(
            Array::new(
                vec![2, 3],
                ArrayData::Float(vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0]),
            )
            .unwrap(),
        );
        assert_eq!(
            r.call("hello", "sum", std::slice::from_ref(&m)),
            Ok(Value::Float(10.5))
        );
        assert_eq!(r.call("hello", "echo", std::slice::from_ref(&m)), Ok(m));
    }
}

#[test]
fn errors() {
    for r in both() {
        assert_eq!(
            r.call("hello", "fail", &[]),
            Err(CallError::Failed("hello was asked to fail".into()))
        );
        assert_eq!(
            r.call("hello", "panic", &[]),
            Err(CallError::Panicked(
                "extension panicked: hello was asked to panic".into()
            ))
        );
        assert_eq!(r.call("hello", "answer", &[]), Ok(Value::Int(42)));
        assert_eq!(
            r.call("hello", "shout", &[Value::Bool(true)]),
            Err(CallError::InvalidArgument("expected text (Char)".into()))
        );
        assert_eq!(
            r.call("hello", "add", &[Value::Int(1)]),
            Err(CallError::Arity {
                function: "hello/add".into(),
                expected: 2,
                got: 1
            })
        );
        assert_eq!(
            r.call("hello", "nope", &[]),
            Err(CallError::UnknownFunction {
                extension: "hello".into(),
                function: "nope".into()
            })
        );
        assert_eq!(
            r.call("bye", "answer", &[]),
            Err(CallError::UnknownExtension("bye".into()))
        );
    }
}

#[test]
fn help_and_functions() {
    for r in both() {
        assert_eq!(
            r.help("hello", "shout").unwrap(),
            "hello/shout : Char -> Char\nThe text in upper case."
        );
        assert_eq!(r.functions("hello").len(), 8);
        assert!(r.help("hello", "nope").is_none());
    }
}

#[test]
fn deactivation() {
    for mut r in both() {
        assert!(r.deactivate("hello"));
        assert_eq!(
            r.call("hello", "answer", &[]),
            Err(CallError::Inactive("hello".into()))
        );
        assert!(!r.deactivate("bye"));
    }
}

#[test]
fn loading_twice_is_refused() {
    let mut r = dynamic();
    assert!(matches!(
        r.load_static(xetal_ext_hello::__xetal_extension::descriptor),
        Err(LoadError::AlreadyLoaded(n)) if n == "hello"
    ));
}

/// A package directory in the temp dir with the given manifest text.
struct Fixture(PathBuf);

impl Fixture {
    fn new(tag: &str, manifest: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xetal-ext-{}-{tag}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("extension.toml"), manifest).unwrap();
        Self(dir)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const HELLO: &str = "[extension]\nname = \"hello\"\nversion = \"0.1.0\"\nabi = 1\nfacade = \"Hello.xtl\"\nlibrary = \"xetal_ext_hello\"\n";

#[test]
fn a_packaged_library_is_found_under_native() {
    let f = Fixture::new("packaged", HELLO);
    let native = f.0.join("native").join(xetal_ext_loader::platform_triple());
    fs::create_dir_all(&native).unwrap();
    let p = Package::open(&f.0).unwrap();
    let built = p.library(&build_dirs()).unwrap();
    fs::copy(&built, native.join(p.library_file())).unwrap();
    let path = p.library(&build_dirs()).unwrap();
    assert!(path.starts_with(&native));
    let mut r = Registry::new();
    r.load_package(&p, &[]).unwrap();
    assert_eq!(r.call("hello", "answer", &[]), Ok(Value::Int(42)));
}

#[test]
fn manifest_problems() {
    let f = Fixture::new("version", &HELLO.replace("0.1.0", "9.9.9"));
    let err = Registry::new()
        .load_package(&Package::open(&f.0).unwrap(), &build_dirs())
        .unwrap_err();
    assert!(
        matches!(
            err,
            LoadError::Mismatch {
                field: "version",
                ..
            }
        ),
        "{err}"
    );

    let f = Fixture::new("abi", &HELLO.replace("abi = 1", "abi = 2"));
    let err = Package::open(&f.0).unwrap_err();
    assert!(
        err.to_string().ends_with("abi 2 is not supported (1 is)"),
        "{err}"
    );

    let f = Fixture::new("unknown", &format!("{HELLO}color = \"blue\"\n"));
    assert!(matches!(
        Package::open(&f.0),
        Err(LoadError::Manifest { .. })
    ));

    let f = Fixture::new("path", &HELLO.replace("\"xetal_ext_hello\"", "\"../evil\""));
    assert!(matches!(
        Package::open(&f.0),
        Err(LoadError::Manifest { .. })
    ));

    let f = Fixture::new(
        "missing",
        &HELLO.replace("xetal_ext_hello", "xetal_ext_missing"),
    );
    let err = Package::open(&f.0)
        .unwrap()
        .library(&build_dirs())
        .unwrap_err();
    match err {
        LoadError::NoLibrary { extension, tried } => {
            assert_eq!(extension, "hello");
            assert_eq!(tried.len(), 2);
        }
        e => panic!("{e}"),
    }

    let missing = std::env::temp_dir().join("xetal-ext-no-such-dir");
    assert!(matches!(
        Package::open(missing),
        Err(LoadError::Manifest { .. })
    ));
}
