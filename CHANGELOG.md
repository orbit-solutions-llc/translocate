# Changelog

## Unreleased

---
## v0.8.1-beta.1 | *2024-12-18*
- Switch to use `@orb/simple-binary-install` package hosted on JSR registry

---
## v0.8.0 | *2024-01-10*
- Added new command line flag `-I`/`--ignored_headings` which allows excluding CSV one or more column headers from processing. e.g.

```sh
translocate -I "en_US,es_ES" ./path/to/file.csv
```

## v0.7.0 | *2024-01-04*
- Only overwrite translation keys when the value is not empty.
- Fix parsing of terminator CLI argument `-t`/`--terminator`.

## v0.6.0 | *2024-01-01*
- Add cli option `-O`/`--output_filename` to output to specific filenames.
- When this option is used, the locales available will be used as directory names.

## v0.5.2 | *2023-12-30*
- Improve crate documentation.

## v0.5.1 | *2023-12-27*
- Republish 0.5.0 without useless csv/tsv files included.

## v0.5.0 | *2023-12-21*
- Don't overwrite translation key if replacing translation is an empty string.
- Add more tests testable.

## v0.4.0 | *2023-12-12*
- Made output directory configurable.
- Modularized app logic to make it more testable.

## v0.3.2 | *2023-12-09*
- Create npm package for use in node or web dev projects.

## v0.3.0 | *2023-12-08*
- Add command line option to trim whitespace from from CSV fields before conversion to JSON.
- Update README.

## v0.2.0 | *2023-10-28*
- Add command line parsing options and update the README.

## v0.1.0 | *2023-10-27*
- Release initial version of translocate.
