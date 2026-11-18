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

## Evaluator values: `crates/xetal-ext-values`

A host that holds a value of the X_eTaL evaluator (`xetal_value::Value`
at `XETAL_COMMIT`) gives it to an extension, and takes the reply back,
with no text in between:

| Function | What it does |
| -------- | ------------ |
| `to_abi(caller, &value)` | the evaluator's value as an ABI value: Int, Float and Bool scalars as they are, a Char vector as text, any other array dense (all Int, all Float, all Bool or all Char), row-major, its shape kept |
| `from_abi(caller, &value)` | the other way: text is a Char vector, an array of rank 0 its one item |
| `mesh(caller, &positions, &triangles, base)` | an N by 3 Float array and a T by 3 Int array as one `Mesh` (corners as `f64`, triangles as `u32` from 0); `Base::One` when the triangles count corners from 1, as X_eTaL does. `mesh_of` takes the evaluator's values |
| `f32s(caller, &numbers, policy)` | Floats as `f32`, only when asked: `Narrow::Rounded` (nearest; refused when not finite, beyond `f32`, or not zero and rounding to zero) or `Narrow::Exact` (only numbers an `f32` holds). `Mesh::positions_f32` is the same for a mesh |
| `Published` | values a running program left for its host: `hook()` goes to the evaluator's output hook, `last()` is the last top-level value, `named(caller, "frame!")` a variable where the last top-level function was made |

Every refusal begins with the caller the host gave, then says what and
where: `snow/chunk: triangles: triangle 2 names corner 3; the corners
are 1 to 2`, `snow/chunk: positions have shape 3 2, not rows of 3`,
`snow/chunk: positions: corner 2 has a coordinate that is not finite
(NaN)`, `Game.xtl draw: a function does not cross the extension
boundary`. Refused by `to_abi`: functions, tuples, boxes, tags, events
and errors, and an array whose items are not all of one kind (an Int
among Floats is named by its place, never converted). An empty array of
numbers crosses as Floats: the evaluator does not say which number an
empty array holds.

Limits, said plainly:

- This is the host's side. A call written in X_eTaL still crosses as
  text (`docs/bridge.md`) until X_eTaL has a native hook
  (`docs/xetal-asks.md`, E1).
- The crate reads the evaluator's own representation, and `Published`
  its environment: neither is an interface X_eTaL promises. Pin this
  repository and X_eTaL together (this repository's `XETAL_COMMIT`), and
  expect `Published` to go when E1 lands.
- Values are copied, once each way; nothing here is zero-copy.
- It builds for `wasm32-unknown-unknown`; it has been run natively
  only.
