# translocate - convert CSV translation files to JSON

**`translocate`** is a high performance (*blazingly fast*) CSV translation file to JSON translation file transformer. It takes `.csv` or `.tsv` files as input, and will output one file for each language listed as a CSV column.

## Requirements for use

There are one main requirements which should be followed for this crate to work optimally:
  - The first line of the input CSV (or TSV) file should be the heading column.

`translocate` has two internal methods used to transform your localized strings into JSON. The faster and more permissive method only requires that a heading line is present. `translocate` automatically tries the faster conversion method and, if it fails, will fall back to the stricter, slower strategy. 

The alternate, stricter method attempts to deserialize input files based on a predefined list of languages. For this alternate mode the language identifiers in the heading should be in the format `xx_YY` or `xx-YY` &mdash; e.g. **en_US** or **en-US**. The very first heading should be named `id`, this will provide the translation keys. The current list of languages is `da_DK`, `de_DE`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `nl_NL`, `pt_BR`, `pt_PT` and `sv_SE`. An example of the heading in a CSV with all supported languages is shown below:

```
id,da_DK,de_DE,en_US,es_ES,fr_FR,it_IT,nl_NL,pt_BR,pt_PT,sv_SE
```

If your particular requirements are not being served, you are encouraged to [make a pull request](https://code.orbitsolutions.dev/orb-it-solutions/translocate/pulls) which adds support. Alternatively you may fork the repository and modify for your specific needs; the license is *quite* permissive.

If you need to do some processing of your CSV before passing to **`translocate`** because of failures during the transformation from CSV to JSON, consider checking out the [qsv](https://crates.io/crates/qsv) or [xsv](https://crates.io/crates/xsv) crates.
