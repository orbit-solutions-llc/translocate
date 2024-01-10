# Release v0.8.0

## *2024-01-10*
- Added new command line flag `-I`/`--ignored_headings` which allows excluding CSV one or more column headers from processing. e.g.

```sh
translocate -I "en_US,es_ES" ./path/to/file.csv
```
