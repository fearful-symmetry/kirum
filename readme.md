# Kirum

Kirum (from Standard Babylonian _Kirûm_ meaning _garden_ or _orchard_) is a conlang CLI utility and library.
Unlike many conlang tools, which allow you to generate words based on phonetic rules, Kirum can generate entire languages, or language families, based on specified etymology. Kirum also takes a "pets not cattle" approach to conlang tooling, allowing users to store and graph the entire history of a language family, down to individual morphemes.

As an example, we can use kirum to graph the (simplified) etymology of the English word _bureaucracy_ using a tree file:

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
    bureaucracy (English) government of bureaus
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
burracracy (English) government of bureaus
```

We can also use a similarly simple one-line change to imagine what would happen if the original latin root was _purra_ instead of _burra_, and re-generate our entire lexicon:
```
kirum render -d examples/bureaucracy line
    cracy (English) rule or government by
    cratia (Latin) power, rule
    cratie (French) rule or government by
    purel (Old French) coarse cloth
    pureau (French) desk with drawers and papers, office
    pureaucracy (English) government of bureaus
    kratia (Greek) power, rule
    purra (Latin) wool
```

This is the core of Kirum's functionality; by storing the relationship between words, instead of words themselves, we can easily make fundamental changes to a conlang, and quickly re-generate the entire lexicon.

### TODO:
- Docs
- fix CSV encoding issues
- word generation from phonetic rulesets?
- IPA support
- higher-level transformations (vowel shifts, etc)
- Templating and variables in lexical definitions
- add rhai scripts to transform
- Add tests for tmpl.rs
- Make cool spinner for background tasks
- Redo Word type, a word should probably be a String + metadata about individual characters
- transforms should be optional
- Some kind of "auto-derive" functionality