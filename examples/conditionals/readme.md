# Generating a Daughter Language

Kirum transforms have `conditional` fields that can be used to selectively apply the given transform.

To run this example, use the include [transform](etymology/example_daughter_transform.json) to generate a daughter language:

```
$ kirum generate daughter -d conditionals -e conditionals/etymology/example_daughter_transform.json -a "Old Exemplum" -n "Middle Exemplum" -o conditionals/tree/middle.j
son

[2023-05-28T18:19:01Z INFO  kirum] wrote daughter Middle Exemplum to conditionals/tree/middle.json

$ render -d conditionals line

    chadein (Middle Exemplum): (Noun) A cultivated plot of earth, garden
    shottin (Middle Exemplum): (Noun) A type of fungi, mushroom
    tvømm (Middle Exemplum): (Adjective) Water that falls from the sky, rain
    vednin (Middle Exemplum): (Noun) Water that falls from the sky, rain
    chade (Old Exemplum): (Noun) A cultivated plot of earth, garden
    shott (Old Exemplum): (Noun) A type of fungi, mushroom
    tvømm (Old Exemplum): (Adjective) Water that falls from the sky, rain
    vedn (Old Exemplum): (Noun) Water that falls from the sky, rain
```

Note that in the transform file we used, the two transforms have conditionals that mean not every word in `Old Exemplum` was transformed when it was carried over to the new language. Because the word `shott` was marked as `archaic: true`, the `letter_replace` vowel change was not applied, and `tvømm` does not have an `in` suffix because it does not match the `"oneof": ["noun", "verb"]` conditional.

## Possible values in a conditional field

The `conditional` json object is arranged as such:

```
    "conditional":{ // 1.
        "pos": { // 2.
            "match":{ // 3.
                "oneof": ["noun", "verb"] //4.
            }
        }
    }
```

1. The base conditional JSON object
2. The lex value to match against. Can be any value in the `lexis` object
3. One of "match" or "not". Note that because "archaic" is a bool, a conditional on "archaic" will just be "true" or "false"
4.  The match "righthand" statement. Can be one of "equals" or "oneof"