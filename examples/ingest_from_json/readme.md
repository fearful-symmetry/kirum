# Ingesting a File From an External Source

Kirum can ingest words from two kinds of external sources: a newline-delimited list of words, and an arbitrary JSON file.

This json file can simply be a list of words, or can be used to minimally graph word etymology. At minimum, an ingested JSON file must be structured as such:

```json

{
    "keys_are": "definitions", // determines if the ingested words will appears as definitions, or words
    "words": [
        // a list of words, or objects defining word relationships
    ]
}
```

The contents of the `words` list can be a structured in a number of ways. `input.json` provides a maximal example:

```json
{
    "keys_are": "definitions",
    "words": [
        "to grab", // will create a lexical entry with the definition "grab"
        {
            "fail": { // will create three words: "fail", with two child words, "failing" and "failure", 
                //defied by the etymological transforms to_do and state_of
                "failing": "!to_do",
                "failure": "!state_of"
            }
        },
        {
            "attack": ["attacking", "attacked"] // will create three words, "attacking" and "attacked", with the etymon "attack"
        },
        {
            "twist": { // will create a root "twist" and all specified derivatives 
                "twistable": {
                    "untwistable": "!negate", // will create the derivative "untwistable" with the transform "negate"
                    "!etymology": "capability", // will create the derivative "twistable" with the etymological transform "capability"
                    "retwistable": "unretwistable" // will make a derivative "retwistable" with a further derivative "unretwistable"
                }
            }
        }
    ]
}
```

In this directory, we have an empty project, which defines a series of etymological rules, and all the specified transforms, but no lexicon.

We can ingest `input.json` with the following command:

``` bash
kirum ingest -d empty_language -o generate=word -o language="example" json input.json
```

The `-o` flags define overrides that will apply to all ingested words, in this case setting the phonetic generator to the key `word`
as defined in `phonetics/rules.json`, and the language of all words to `example`.

After this, we can generate our dictionary from our imported words:

```bash
kirum render -d empty_language line
```