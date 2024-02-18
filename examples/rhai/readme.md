# Using the Rhai scripting language to transform words

Kirum supports [Rhai](https://github.com/rhaiscript/rhai) for transforming words in a language tree using a simple scripting language. This allows for more complex transforms based on string manipulation and conditional logic:

```rhai
// if the etymon's language is "mylang", remove all instances of the letter "t"
// and add the post fix "ah" if the word also starts with "el".
if language == "mylang" {
    updated.remove("t");
    if updated.starts_with("el"){
        updated = updated + "ah"
    }
}
```

Rhai documentation can be found [here](https://rhai.rs/book/ref/index.html).

To use a rhai script, specify it as a transform:
```json
        "from-root" : {
            "transforms": [
                {"rhai_script": {"file": "rhai/string_transform.rhai"}}
            ]
        },
```

As demonstrated in [string_transform.rhai](rhai/string_transform.rhai), the the Rhai script exports a number of variables
that can be used in a script to transform a word selectively based on the word's associated metadata.

To render the test, run:

```bash
kirum render -d ./ line
```