# translocate - convert CSV translation files to JSON

**`translocate`** is a high performance (it's *blazingly* fast) CSV translation to JSON translation file transformer. It takes `.csv` or `.tsv` files as input, and will output one file for each language listed as a CSV column.

## Requirements for use

There are some requirements for this crate to work properly:
  1. The input CSV (or TSV) file has a heading column.

There are currently two internal methods used to transform your localized strings into JSON. The *fast* way only requires that CSV headers are present. The alternate method deserializes data based on a predefined list of languages. 
  1. For this alternate mode the language indentifiers in the CSV should be in the format `xx_YY` or `xx-YY` &mdash; e.g. **en_US**.
  2. This list includes the following languages: `da_DK`, `de_DE`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `nl_NL`, `pt_BR`, `pt_PT` and `sv_SE`.

If any of these requirements can not be met, interested users are encouraged to [make a pull request](https://code.orbitsolutions.dev/orb-it-solutions/translocate/pulls). Alternatively you may fork the repository and modify for your specific needs, as the license is *quite* permissive.

If you need to do some processing of your CSV before passing to **`translocate`**, consider checking out the [qsv](https://crates.io/crates/qsv) or [xsv](https://crates.io/crates/xsv) crate.
