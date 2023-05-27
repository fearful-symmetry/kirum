# Generating a Daughter Language

Kirum has the ability to generate a daughter language from an existing language using only an additional etymology file.
For example, the language here has a simple three-word lexicon, with the language `Old Exemplum`:
```
$ kirum render -d examples/generate_daughter line 
    chade (Old Exemplum): (Noun) A cultivated plot of earth
    shott (Old Exemplum): (Noun) A type of fungi
    vedn (Old Exemplum): (Noun) Water that falls from the sky
```

We can use the etymology file [here](etymology/example_daughter_transform.json), to generate a new language, `Middle Exemplum`, which will replace all instances of the vowel `e` with `ai` and de-double the consonant `t`:
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

