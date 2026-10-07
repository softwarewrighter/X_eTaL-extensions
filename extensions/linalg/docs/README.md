# linalg

Numerical linear algebra for X_eTaL, through nalgebra: solving systems,
inverses, determinants, least squares, symmetric eigenvalues and the
singular value decomposition -- the algorithms an array language should
call, not reinvent. Matrices are Float arrays, rows by columns.

- Package: `extensions/linalg/` (`extension.toml`; library
  `xetal_ext_linalg`: `libxetal_ext_linalg.dylib` on macOS, `.so` on
  Linux)
- Facade: `lib/Linalg.xtl`, recommended alias `la:`
- Native crates: `nalgebra` 0.33

## From X_eTaL

```
"la:" u_se< "Linalg"
a := 2 2 r_eshape 2.0 1.0 1.0 3.0
a la:s_olve 3.0 5.0                       # 0.8 1.4
la:d_et a                                 # 5.0
a '+ '* i_nner la:i_nverse a              # the identity
la:e_ig a                                 # eigenvalues, ascending
w := 2 3 r_eshape 3.0 2.0 2.0 2.0 3.0 -2.0
la:s_vdS w                                # 5.0 3.0
```

| Export | Type | What |
| ------ | ---- | ---- |
| `a la:s_olve b` | `(Num a, Num b) => a -> b -> Float` | x with a x = b: a square and not singular; b a vector or a matrix of columns |
| `la:i_nverse a` | `Num a => a -> Float` | the inverse of a square matrix |
| `la:d_et a` | `Num a => a -> Float` | the determinant |
| `a la:l_stsq b` | `(Num a, Num b) => a -> b -> Float` | the x minimizing the length of a x - b (least squares, by the SVD; the shortest x when several fit) |
| `la:e_ig a` | `Num a => a -> Float` | a symmetric matrix's eigenvalues, ascending |
| `la:e_igVecs a` | `Num a => a -> Float` | their eigenvectors, one column each, in the same order |
| `la:s_vdS a` | `Num a => a -> Float` | the singular values, descending |
| `la:s_vdU a`, `la:s_vdV a` | `Num a => a -> Float` | the singular vectors, one column each: a = U diag(s) V' (thin: r = the smaller side) |

Native functions: `solve`, `inverse`, `det`, `lstsq`, `eigsym`,
`eigvecs`, `svd_s`, `svd_u`, `svd_v` (`just list`).

A singular matrix (a pivot below 1e-12 of the largest entry) is an
error for `s_olve` and `i_nverse`; `l_stsq` handles it. A matrix with
a NaN or an infinity is refused. Each SVD function computes the
decomposition again: call each once and keep the results.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The Rust tests check known answers (a 2 by 2 system, its inverse, a
determinant, a line fitted exactly, the eigenvectors of a 2 by 2, an
SVD rebuilding its matrix) and the errors. The reg-rs test
`linalg-cross-check` computes the same small cases in pure X_eTaL --
Cramer's rule, the 2 by 2 eigenvalue formula, inverse times the matrix,
U diag(s) V' -- and prints 1 for each that agrees to 1e-9.

## Demos

The photo lab (in the image extension) compresses a photo with
`la:s_vdU`, `la:s_vdS` and `la:s_vdV`.
