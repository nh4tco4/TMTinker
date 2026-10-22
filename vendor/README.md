`ron/` contains RON 0.11.0 from crates.io (MIT OR Apache-2.0), patched through
the root Cargo.toml for eframe 0.33's native persistence.

The only source change is in `Parser::escaped_byte_buf` in `src/parse.rs`:
scan for a quote or a backslash together, instead of repeatedly searching for
the closing quote after every escape. A saved TU4 source with many newlines
and few quotes otherwise takes quadratic time to load before the window opens.
The storage format is unchanged; existing project libraries remain readable.

The package manifest retains the upstream package metadata, features and runtime
dependencies. Upstream examples and development dependencies are omitted.
Regression coverage is in `tests/native_storage.rs`.
