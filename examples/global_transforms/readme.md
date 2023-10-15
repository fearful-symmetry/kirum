# Using Global Transforms 

In addition to specifying transforms in word etymology, transforms can be specified at a global level, where they will be applied to all words that meet the given match statements.

Unlike an etymon-level transform, global transforms can take a match statement for the `lexis`, the word targeted by the transform, as well as `etymon`, which will match the first upstream etymon of the given lexis.

After looking at the example in `globals.json`, you can run the example file with 

`kirum render -d ./ line`