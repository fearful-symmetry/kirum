# Using output templates to generate asciidoc

Kirum has the ability to output a rendered lexicon using a template file, as well as the ability to use [rhai](https://github.com/rhaiscript/rhai) scripts as template helpers. 

This example uses a simple handlebars template and rhai script to generate an asciidoc dictionary of the included language. Here, the rhai script shortens the part of speech value from a whole world (`noun`) to the first letter, (`n`).

```
$ kirum -o rendered.asciidoc render -d templates template -t templates/tmpl/lexicon.hbs -r templates/tmpl/shorten_pos.rhai

$ cat rendered.asciidoc

== Lexicon

_essemple_ n. 'model, example'.

_exemplum_ n. 'an instance, model, example'.

_emere_ v. 'To buy, remove'.
```