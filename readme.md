# Kirum

![ci](https://github.com/fearful-symmetry/kirum/actions/workflows/rust.yml/badge.svg)

Kirum (from Standard Babylonian _Kirûm_ meaning _garden_ or _orchard_) is a conlang CLI utility and library.
Unlike many conlang tools, which allow you to generate lexicons based on phonetic rules, Kirum can generate entire languages, and manage whole language families, based on specified etymology. Kirum also takes a "pets not cattle" approach to conlang tooling, allowing users to store and graph the entire history of a language family, down to individual morphemes.

Kirum is a work in progress, and should be considered alpha software. Major features are currently planned, including the ability to generate languages/words from phonetic rulesets, and IPA support.

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


## The structure of `tree` and `etymology` files

`kirum` generates languages from two files: A tree file, which contains a lexicon of words, stems, roots, etc, and an etymology file, which contains data on the transforms between words. The transform files can also contain conditional statements that determine if a transform should be applied to a word.
