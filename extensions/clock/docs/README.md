# clock

Wall-clock and monotonic time, which X_eTaL does not have (`docs/xetal-asks.md`
in the repository, E4): timestamps for programs, and timing for
benchmarks -- including the cost of the bridge itself.

- Package: `extensions/clock/` (`extension.toml`; library
  `xetal_ext_clock`: `libxetal_ext_clock.dylib` on macOS, `.so` on Linux)
- Facade: `lib/Clock.xtl`, recommended alias `ck:`
- Native crates: none beyond the SDK (std's `SystemTime` and `Instant`)

## From X_eTaL

```
"ck:" u_se< "Clock"
ck:i_so @                          # 2026-10-02T23:14:05.123Z
ck:i_soOf 951782400                # 2000-02-29T00:00:00.000Z
'r_ange ck:t_ime 1000000           # milliseconds to make a million-element vector
```

| Export | Type | What |
| ------ | ---- | ---- |
| `ck:n_ow @` | `Unit -> Float` | seconds since 1970-01-01 UTC, with the fraction |
| `ck:i_so @` | `Unit -> Char` | the current time, ISO 8601 UTC, to the millisecond |
| `ck:i_soOf t` | `Num a => a -> Char` | Unix seconds as ISO 8601 UTC (years 0000 to 9999; otherwise an error) |
| `ck:m_illis @` | `Unit -> Float` | milliseconds on a monotonic clock (never goes back) |
| `'f_ ck:t_ime x` | `(a -> b) -> a -> Float` | milliseconds `f_ x` takes (written in X_eTaL over `m_illis`) |

Native functions: `now`, `iso`, `iso_of`, `millis` (`just list`).

## Demos

- `demos/bridge-cost.xtl`: what the text bridge costs. Run with
  every extension loaded (it uses hello too):
  `xetal-x --ext extensions run extensions/clock/demos/bridge-cost.xtl`.
  On an Apple M-series laptop, release build: about 220,000 calls per
  second, 2.6 million Floats per second out to Rust and back, and a
  100,000-element sum takes about 21 ms in X_eTaL and 20 ms in Rust
  over the bridge (most of it formatting the numbers as text).

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (ISO dates, descriptor) and reg-rs tests
just list       # the functions as xetal-x sees them
```

Real times differ on every run, so the tests check what is fixed:
conversions of given times, formats, order and kinds; the demo's
golden masks its numbers.
