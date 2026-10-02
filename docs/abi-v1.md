# Native extension ABI V1

How an X_eTaL extension and its host talk: a C-compatible layout,
defined in `crates/xetal-ext-abi`. Adapted from sw-ml-study's
demo-extensions ABI V1 (same author, MIT), reduced to X_eTaL's types:
no nil, bytes, handles or records; Char arrays; arity 0 to 2; an X_eTaL
type signature on every function.

## The entry point

An extension is a shared library (`.dylib` on macOS, `.so` on Linux)
that exports one symbol:

```
const ExtensionDescriptorV1 *xetal_extension_v1(void);
```

It returns a pointer to a descriptor that lives as long as the library
is loaded.

## Records

All records are `#[repr(C)]`; tags and codes are `u32`; reserved fields
must be zero. Sizes are pinned by a test on 64-bit targets.

| Record | Fields | Size |
| ------ | ------ | ---- |
| `AbiSlice` | `data: *const u8`, `len: usize` (empty: null and 0) | 16 |
| `ExtensionDescriptorV1` | `struct_size: u32`, `abi_version: u32` (1), `name`, `version` (slices), `functions: *const FunctionDescriptorV1`, `function_count: usize`, `reserved: u64` | 64 |
| `FunctionDescriptorV1` | `name`, `arity: u32` (0, 1 or 2), `reserved: u32`, `signature` (the X_eTaL type, e.g. `Char -> Char`), `doc` (one line, may be empty), `invoke` (the trampoline, required) | 64 |
| `AbiValue` | `tag: u32`, `reserved: u32`, `payload` (a union) | 24 |
| `AbiArrayView` | `dtype: u32`, `rank: u32`, `shape: *const usize`, `data` (a slice of bytes) | 32 |
| `AbiErrorV1` | `code: u32`, `reserved: u32`, `message` (UTF-8 slice) | 24 |

`struct_size` and `abi_version` come first, so a host reads them before
anything else and rejects a layout it does not know.

## Values

| Tag | Name | Payload | X_eTaL |
| --- | ---- | ------- | ------ |
| 1 | Bool | `u8`, 0 or 1 | a Bool scalar |
| 2 | Int | `i64` | an Int scalar |
| 3 | Float | `f64` | a Float scalar |
| 4 | Text | UTF-8 slice | a Char vector |
| 6 | Array | `*const AbiArrayView` | an array of any rank |

Tags 0, 5, 7 and 8 are reserved (demo-extensions uses them for nil,
bytes, handles and records) and rejected.

An array is dense, contiguous and row-major: `rank` axes (0 to 9; rank
0 is a scalar held as an array), the shape's product elements, each
`dtype`:

| dtype | Element | Bytes |
| ----- | ------- | ----- |
| 1 | Bool | 1 (0 or 1) |
| 2 | Int | 8 (`i64`, native order) |
| 4 | Float | 8 (`f64`, native order) |
| 5 | Char | 4 (a Unicode scalar value as `u32`, native order) |

Bounds: at most 64 Mi elements per value (text at most 256 MiB);
descriptor text at most 16 KiB; at most 1,024 functions.

## Calls

```
u32 invoke(const AbiValue *arguments, usize argument_count,
           AbiValue *output, AbiErrorV1 *error);
```

The return value is a code: 0 Ok (the result is in `*output`), 1
InvalidArgument, 2 ExtensionFailure, 3 Panic (the message is in
`*error`). X_eTaL calls a function with one argument (monadic) or two
(dyadic, left then right); arity 0 is a function that ignores its
argument (called with `@`).

Ownership: the arguments are the host's, valid during the call. The
output and error point into storage the extension keeps until the next
call on the same thread; the host copies them before calling again.

## Safety

- Validation (`validate_descriptor`) and value decoding
  (`copy_foreign_value`, `copy_foreign_error`) are the only code that
  reads foreign pointers. They check every count, length and bound
  before reading, and copy everything into owned Rust values: the host
  keeps no extension pointer except the trampolines, which are valid
  while the library stays loaded (the loader keeps it loaded).
- A Rust panic never crosses `extern "C"`: the SDK's trampolines catch
  it (`catch_extension_call`) and return code 3.
- Rejected: wrong size or version, non-zero reserved fields, a length
  without data, invalid UTF-8, empty names or signatures, an arity above
  2, a missing trampoline, a duplicate function name, an unknown tag or
  dtype, a Bool other than 0 or 1, a Char that is not a scalar value, a
  rank above 9, a shape whose product disagrees with the data.
