# Creating new words with phonetic rules

In addition to creating word based on etymological rules and word relationships, Kirum can also generate
words from base phonetic rulesets, without any pre-existing etymology.

To generate phonetic rules, a Kirum project must have one or more JSON rule files under `phonetics/` in the root project directory. These files are formatted as such:

```json
{
  "groups": {
    "V": [
      "e",
      "a"
    ],
    "S": [
      "VC",
      "CCV",
      "VyV"
    ],
    "C": [
      "x",
      "m",
      "p",
      "l"
    ]
  },
  "lexis_types": {
    "word_rule": [
      "SSS",
      "SCSS"
    ]
  }
}

```

This phonetic file is divided into two maps:
- `groups`: breaks down possible groups of letters and consonants. The key of a group can be any uppercase unicode character, the values of an individual group can be any unicode value, or any uppercase group key.
In the above example, `V` are the language's possible vowels, `S` are the possible syllables, and `C` are
the possible consonants.
- `lexis_rules`: are the possible words that are derived from the specified group rules.

To generate a word from a set of specified phonetic rules, simply add the given `lexis_types` value to
the lexis's `generate` field:
```json
    "latin_verb": {
      "type": "word",
      "generate": "word_rule",
      "language": "Latin",
      "definition": "To buy, remove",
      "part_of_speech": "verb",
      "archaic": true
    }
```

Note that the generator will not apply a new word if the lexis has both a `generate` and `word` field.
