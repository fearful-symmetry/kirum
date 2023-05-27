# Kirum

Kirum (from Standard Babylonian _Kirûm_ meaning _garden_ or _orchard_) is a conlang CLI utility and library.
Unlike many conlang tools, which allow you to generate words based on phonetic rules, Kirum can generate entire languages, or language families, based on specified etymology. Kirum also takes a "pets not cattle" approach to conlang tooling, allowing users to store and graph the entire history of a language family, down to individual morphemes.

Kirum is a work in progress, and should be considered alpha software. Major features are currently planned, including the ability to generate languages/words from phonetic rulesets, and IPA support.

## Examples

### Graphing and changing the etymology of a word

We can use kirum to graph the (simplified) etymology of the English word _bureaucracy_ using a tree file:

```json
{
    "words": {
        "completed-word": {
            "language":"English", "definition": "government of bureaus",
            "etymology": {
                "etymons":[
                    {"etymon": "french-prefix", "transforms": ["loanword"], "agglutination_order": 0},
                    {"etymon": "english-dem-suffix", "transforms": ["loanword"],"agglutination_order": 1}
                ]
            }
        },
        "french-prefix": {
            "language": "French", "definition": "desk with drawers and papers, office",
            "etymology": {
                "etymons": [{"etymon": "old-french-prefix", "transforms": ["from-old-french"]}]
            }
        },
        "old-french-prefix": {
            "language": "Old French", "definition": "coarse cloth",
            "etymology": {"etymons": [{"etymon": "latin-cloth", "transforms": ["from-latin", "from-latin-old-fr-diminutive"]}]
            }
        },
        "latin-cloth": {
            "word": "burra", "language": "Latin", "definition": "wool"
        },
        "english-dem-suffix": {
            "language": "English", "definition": "rule or government by",
            "etymology": {"etymons": [{"etymon": "french-dem-root", "transforms": ["english-from-french"]}]
            }
        },
        "french-dem-root": {
            "language": "French", "definition": "rule or government by",
            "etymology": {"etymons": [{"etymon": "latin-dem-root", "transforms": ["from-latin"]}]
            }
        },
        "latin-dem-root": {
            "language": "Latin", "definition": "power, rule",
            "etymology": {"etymons": [{"etymon": "greek-stem", "transforms":["from-greek"]}]}
        },
        "greek-stem": {
            "word": "kratia", "language": "Greek", "definition": "power, rule"
        }
    }
}
```
And an etymology file:
```json
{
    "transforms": {
        "loanword": {
            "transforms": ["loanword"]
        },
        "from-old-french": {
            "transforms": [
                {"letter_remove": {"letter": "l", "position": "last"}},
                {"letter_replace": {"letter": {"old":"e", "new":"eau"},"replace": "all"}}
            ]
        },
        "from-latin": {
            "transforms": [
                {"dedouble": {"letter": "r", "position": "first"}},
                {"letter_replace": {"letter": {"old": "a", "new":"e"}, "replace": "last"}}
            ]
        },
        "english-from-french": {
            "transforms": [
                {"match_replace": {"old": "tie", "new": "cy"}}
            ]
        },
        "from-latin-old-fr-diminutive": {
            "transforms": [
                {"postfix": {"value": "l"}} 
            ]
        },
        "from-greek": {
            "transforms": [
                {"letter_replace": {"letter": {"old": "k", "new": "c"}, "replace": "all"}}
            ]
        }
    }
}
```

We can then use `kirum` to render the complete lexicon for our language etymology:
```
$ kirum render -d examples/bureaucracy line
    burel (Old French) coarse cloth
    bureau (French) desk with drawers and papers, office
    bureaucracy (English) government defined by red tape, paperwork, officialism
    cracy (English) rule or government by
    cratia (Latin) power, rule
    cratie (French) rule or government by
    burra (Latin) wool
    kratia (Greek) power, rule
```

We can then see our target word, _bureaucracy_, along with all intermediate words. Note that in the above tree file, the only words that were hard-defined in code are the Latin root _burra_ and the Greek root _Kratia_; all other words are derived based on the logic specified in the etymology JSON. This means we can make fundamental changes to the etymology or structure of a word with relatively little changes. For example, if we wanted the word _bureaucracy_ to come from the original Latin root, instead of Latin by way of French, we just need to change a single line in the tree file:
```json
{"etymon": "latin-cloth", "transforms": ["loanword"], "agglutination_order": 0},
```

Our above `render` command will then generate:
```
burracracy (English) government defined by red tape, paperwork, officialism
```

We can also use a similarly simple one-line change to imagine what would happen if the original latin root was _purra_ instead of _burra_, and re-generate our entire lexicon:
```
kirum render -d examples/bureaucracy line
    cracy (English) rule or government by
    cratia (Latin) power, rule
    cratie (French) rule or government by
    purel (Old French) coarse cloth
    pureau (French) desk with drawers and papers, office
    pureaucracy (English) government defined by red tape, paperwork, officialism
    kratia (Greek) power, rule
    purra (Latin) wool
```

This is the core of Kirum's functionality; by storing the relationship between words, instead of words themselves, we can easily make fundamental changes to a conlang, and quickly re-generate the entire lexicon.


### Generating a Daughter Language

Kirum has the ability to generate a daughter language from an existing language using only an additional etymology file.
For example, the language under `examples/generate_daughter` has a simple three-word lexicon, with the language `Old Exemplum`:
```
$ kirum render -d examples/generate_daughter line 
    chade (Old Exemplum): (Noun) A cultivated plot of earth
    shott (Old Exemplum): (Noun) A type of fungi
    vedn (Old Exemplum): (Noun) Water that falls from the sky
```

We can easily crete a new language using the etymology file in `examples/generate_daughter/transforms/generate_daughter_transform.json`, to generate a new language, `Middle Exemplum`, which will replace all instances of the vowel `e` with `ai` and de-double the consonant `t`:
```
$ kirum generate daughter -d examples/generate_daughter -a "Old Exemplum" -n "Middle Exemplum" -e examples/generate_daughter/etymology/example_daughter_transform.json -o examples/generate_daughter/tree/middle_exemplum.json
[2023-05-27T01:01:17Z INFO  kirum] wrote daughter Middle Exemplum to examples/generate_daughter/tree/middle_exemplum.json

$ kirum render -d examples/generate_daughter line
    chadai (Middle Exemplum): (Noun) A cultivated plot of earth
    shot (Middle Exemplum): (Noun) A type of fungi
    vaidn (Middle Exemplum): (Noun) Water that falls from the sky
    chade (Old Exemplum): (Noun) A cultivated plot of earth
    shott (Old Exemplum): (Noun) A type of fungi
    vedn (Old Exemplum): (Noun) Water that falls from the sky
```


## The structure of `tree` and `etymology` files

`kirum` generates languages from two files: A tree file, which contains a lexicon of words, stems, roots, etc, and an etymology file, which contains data on the transforms between words. The transform files can also contain conditional statements that determine if a transform should be applied to a word.
