# digest

SHA-256 and CRC-32 for X_eTaL, of text and of files: to check that
content is what it should be -- a downloaded feed against the digest
recorded when it was saved, a file against a checksum published beside
it.

- Package: `extensions/digest/` (`extension.toml`; library
  `xetal_ext_digest`: `libxetal_ext_digest.dylib` on macOS, `.so` on
  Linux)
- Facade: `lib/Digest.xtl`, recommended alias `dg:`
- Native crates: `sha2`, `crc32fast`; `tempfile` for tests

## From X_eTaL

```
"dg:" u_se< "Digest"
dg:s_ha256 "abc"                        # ba7816bf...f20015ad
dg:c_rc32 "123456789"                   # 3421780262 (CB F4 39 26)
want := "7bdaaa1a53ee632e2b0f9e76e3c30127bd9bc6a477b9c7da6ca19e7c90a17e0c"
(dg:s_ha256File "tests/data/quakes.csv") m_atch want     # 1: unchanged
```

| Export | Type | What |
| ------ | ---- | ---- |
| `dg:s_ha256 t` | `Char -> Char` | the SHA-256 of a text (its UTF-8 bytes), as 64 lower-case hex digits |
| `dg:s_ha256File path` | `Char -> Char` | the SHA-256 of a file |
| `dg:c_rc32 t` | `Char -> Int` | the CRC-32 of a text (IEEE, as zip, gzip and PNG use) |
| `dg:c_rc32File path` | `Char -> Int` | the CRC-32 of a file |

Native functions: `sha256`, `sha256_file`, `crc32`, `crc32_file`
(`just list`).

Files are named by paths under the working directory (or
`XETAL_DIGEST_ROOT`): relative, with no `..`. They are read in 64 KiB
pieces, so their size does not matter.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The Rust tests check the standard vectors (the empty text, `abc`,
`123456789`, a non-ASCII letter), a file larger than one piece against
the same text, and the confinement. The reg-rs tests pin the facade's
types, the vectors from X_eTaL, a file against its recorded SHA-256,
and the error for a path outside.

## Demos

The earthquake demo (in the http extension) checks its saved copy of
the feed with `dg:s_ha256File`.
