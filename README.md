# harfruzz
![Build Status](https://github.com/harfbuzz/harfruzz/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/harfruzz.svg)](https://crates.io/crates/harfruzz)
[![Documentation](https://docs.rs/harfruzz/badge.svg)](https://docs.rs/harfruzz)

`harfruzz` is a fork of [`rustybuzz`](https://docs.rs/rustybuzz) to explore porting from `ttf-parser` to 
[`read-fonts`](https://docs.rs/read-fonts) to avoid shipping (and maintaining)
multiple implementations of core font parsing for [`skrifa`](https://docs.rs/skrifa) consumers.
Further context in https://github.com/googlefonts/fontations/issues/956.

`rustybuzz` is a complete [harfbuzz](https://github.com/harfbuzz/harfbuzz)'s
shaping algorithm port to Rust.

Matches `harfbuzz` [v11.2.1](https://github.com/harfbuzz/harfbuzz/releases/tag/11.2.1).

## Why?

https://github.com/googlefonts/oxidize outlines Google Fonts motivations to try to migrate font
production and consumption to Rust.

## Conformance

The following conformance issues need to be fixed:

* harfruzz does not yet fully pass the harfbuzz shaping or fuzzing tests
* Malformed fonts will cause an error
   * HarfBuzz uses fallback/dummy shaper in this case
* No Arabic fallback shaper (requires subsetting)
* `avar2` as well as other parts of the boring-expansion-spec are not supported yet.

## Major changes

- No font size property. Shaping is always using UnitsPerEm. You should scale the result manually.
- Most of the TrueType and Unicode handling code was moved into separate crates.
- harfruzz doesn't interact with any system libraries and must produce exactly the same
  results on all OS'es and targets.
- `mort` table is not supported, since it's deprecated by Apple.
- No `graphite` library support.

## Performance

At the moment, performance isn't that great. We're 1.5-2x slower than harfbuzz.

See [benches/README.md](./benches/README.md) for details.

## Notes about the port

harfruzz is not a faithful port.

harfbuzz (C++ edition) can roughly be split into 6 parts: 

1. shaping, handled by harfruzz
1. subsetting, (`hb-subset`) moves to a standalone crate, [klippa](https://github.com/googlefonts/fontations/tree/main/klippa)
1. TrueType parsing, handled by [`read-fonts`](https://docs.rs/read-fonts)
1. Unicode routines, migrated to external crates
1. custom containers and utilities (harfbuzz doesn't use C++ std), reimplemented in [`fontations`](https://github.com/googlefonts/fontations) where appropriate (e.g. int set)
1. glue for system/3rd party libraries, just gone

## Lines of code

You can find the "real" code size (eliminating generated code) using:

```sh
tokei --exclude hb/ot_shaper_vowel_constraints.rs \
      --exclude '*_machine.rs' --exclude '*_table.rs' src
```

## Future work

Since the port is finished, there is not much to do other than syncing it with
a new harfbuzz releases. However, there is still lots of potential areas of improvement:

- **Wider test coverage**: Currently, we only test the result of the positioned glyphs in the shaping output.
We should add tests so that we can test other parts of the API as well, such as glyph extents and glyph flags.
- **Fuzzing against harfbuzz**: While `rustybuzz` passes the whole `harfbuzz` test suite, this does not mean that
output will always be 100% identical to `harfbuzz`. Given the complexity of the code base, there are bound to be
other bugs that just have not been discovered yet. One potential way of addressing this issue could be to create a
fuzzer that takes random fonts, and shapes them with a random set of Unicode codepoints as well as
input settings. In case of a discovered discrepancy, this test case could then be investigated and once the
bug has been identified, added to our custom test suite. On the one hand we could use the Google Fonts font
collection for this so that the fonts can be added to the repository,
but we could also just use MacOS/Windows system fonts and only test them in CI, similarly
to how it's currently done for AAT in `harfbuzz`.
- **Performance**: `harfbuzz` contains tons of optimization structures
(accelerators and caches) which have not been included in `rustybuzz`. As a result of this,
performance is much worse in many cases, as mentioned above (although in the grand scheme of things
`rustybuzz` is still very performant), but the upside of excluding all those optimizations is that
the code base is much simpler and straightforward. This makes it a lot easier to backport new changes
(which already is a very difficult task). Now that we are back in sync with `harfbuzz`, we can consider
attempting to port some of the major optimization improvements from `harfbuzz`, but we should do so carefully
to not make it even harder to keep the code bases in sync.
- **Code alignment**: `rustybuzz` tries its best to make the code base look like a 1:1 C++ to Rust translation,
which is the case for most parts of the code, but there also are many variations (most notable in AAT) that arise
from the fact that a lot of the C++ concepts are not straightforward to port. Nevertheless, there probably still
are parts of the code that probably could be made more similar, given that someone puts time into looking into that.

All of this is a lot of work, so contributions are more than welcome.

## Safety

The library is completely safe.

We do have one `unsafe` to cast between two POD structures, which is perfectly safe.
But except that, there are no `unsafe` in this library and in most of its dependencies
(excluding `bytemuck`).

## Alternatives

- [harfbuzz_rs](https://crates.io/crates/harfbuzz_rs) - bindings to the actual harfbuzz library.
  As of v2 doesn't expose subsetting and glyph outlining, which harfbuzz supports.
- [allsorts](https://github.com/yeslogic/allsorts) - shaper and subsetter.
  As of v0.6 doesn't support variable fonts and
  [Apple Advanced Typography](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6AATIntro.html).
  Relies on some unsafe code.
- [swash](https://github.com/dfrg/swash) - Supports variable fonts, text layout and rendering.
  No subsetting. Relies on some unsafe code. As of v0.1.4 has zero tests.

## License

`harfruzz` is licensed under the **MIT** license.

`harfbuzz` is [licensed](https://github.com/harfbuzz/harfbuzz/blob/main/COPYING) under the **Old MIT**
