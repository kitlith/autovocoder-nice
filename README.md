# autovocoder (nice-plug wrapper)

The [original plugin](https://github.com/hotspoons/autovocoder) is good,
I wanted to share it with my friends who use Windows and had a bad time.
(LV2 is not well supported on Windows.)
This repo presents the same plugin but as VST3 and CLAP instead, using the nice-plug framework.

## Build

```
cargo xtask bundle -p autovocoder-nice --release
```

## License

(matching the original)

MIT OR Apache-2.0 — pick whichever you prefer.
