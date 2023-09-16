# Kirum

![ci](https://github.com/fearful-symmetry/kirum/actions/workflows/rust.yml/badge.svg)

Kirum (from Standard Babylonian _Kirûm_ meaning _garden_ or _orchard_) is a conlang CLI utility and library.
Unlike many conlang tools, which allow you to generate lexicons based on phonetic rules, Kirum generates entire languages and language families based on specified etymology. Kirum started as a way to enable easy iteration on a conlang; instead of a blind find-and-replace operation or carefully scrolling through documentation, phonology and conjugation rules can be changed in a single place and then "trickled down" via etymology rules.
The [bureaucracy](examples/bureaucracy/) example demonstrates this by generating the modern English word _bureaucracy_ using only the Latin word _burra_ and the Greek root _kratia_.

Kirum is a work in progress, and should be considered alpha software.

## Getting Started

To create your first project, simply run `kirum new [NAME]`:
```
$ kirum new my_first_project
[2023-05-27T19:57:10Z INFO  kirum] created new project my_first_project
```

This will create a basic project file under a `my_first_project` directory. From there on, you can render your project to a lexicon:

```
$ kirum render -d my_first_project/ line
    essemple (Old French) model, example
    exemplum (Latin): (Noun) an instance, model, example
    emere (Latin): (Verb) To buy, remove
```


## Examples

The [`examples`](examples) directory has a number of projects:

- [bureaucracy](examples/bureaucracy/) - A basic example that demonstrates how to use etymology graphs to make changes to the history of a word.
- [generate_daugher](examples/generate_daughter/) - An example of how to use the `generate` subcommand to create a daughter language from a parent language.
- [templates](examples/templates/) - Using a handlebars template to output an asciidoc dictionary.
- [conditionals](examples/conditionals/) - Using conditional statements in transforms.
- [phonetic_rules](examples/phonetic_rules/) - Using Kirum's phonetic rulesets to generate words.
- [ingest_from_json](examples/ingest_from_json/) - Ingesting words into a language project from a JSON or newline-delimited text file.


## The structure of a Kirum project

`kirum` generates languages from a number of files, contained in separate `tree` and `etymology` directories: Tree files contain a lexicon of words, stems, roots, etc, and etymology files contain data on the transforms between words. The transform files can also contain conditional statements that determine if a transform should be applied to a word. An optional `phonetics` directory also allows for generating words from phonetic, as opposed to etymological, rules.

### Lexis objects

A Tree file is a JSON object of `Lexis` objects, a maximal example of which is presented below:

```json
    "latin_example": {
      "type": "word", // A user-supplied tag. Can be any value.
      "word": "exemplum", // The actual lexical word. If not supplied, kirum will attempt to derive it based on etymology
      "language": "Latin", // Can be any user-supplied value
      "generate": "word_rules", // An optional tag that will generate the word from phonetic rules, see examples/phonetic_rules
      "definition": "an instance, model, example",
      "part_of_speech": "noun", // Optional. Must be one of Noun, verb, or adjective.
      "etymology": {
        "etymons": [
          {
            "etymon": "latin_verb", // The key name of another lexis in the Kirum project
            "transforms": [
              "latin-from-verb" // the key name of a transform
            ]
          }
        ]
      },
      "archaic": true, //optional. Used only for sorting and filtering.
      "tags": [ // optional, user-supplied tags.
        "example",
        "default"
      ],
      "derivatives": [ // The optional derivatives field works as syntactic sugar, allowing users to specify derivative words within the object of the etymon, as opposed to as a separate JSON object.
        {
          "lexis": { // Identical to the `lexis` structure of the parent lexis.
            "language": "Old French",
            "definition": "model, example",
            "part_of_speech": "noun",
            "archaic": true
          },
          "transforms": [
            "of-from-latin"
          ]
        }
      ]
    },
```

### Transform objects

A transform object specifies the relationship between words. Transform files are a JSON object of `Transform` objects, an example of which is below:
```json
        "vowel-o-change":{
            "transforms":[ // a list of individual transform functions. See below for available transforms
                {
                    "letter_replace":{
                        "letter": {"old": "e", "new":"ai"},
                        "replace": "all"
                    }
                }
            ],
            "conditional":{// Optional. The transform will only be applied if the conditional evaluates to true
                "pos": { // will match against the `part_of_speech` field of the Lexis object
                    "match":{
                        "equals": "noun" // The `part_of_speech` field must be equal to `noun`. 
                    }
                }
            }
        }
```

A complete list of available transform types can be found in the [transforms.rs file](libkirum/src/transforms.rs).