// kirum exports the word you want to transform as `lemma_string`

let updated = lemma_string;
updated.replace("k", "c");

// Note that the metadata exported into the Rhai script is the metadata of
// the word youre *transforming from*, not the word you're *transforming into*.

// Kirum also exports the part of speech as `pos`
// see libkirum/src/word.rs for possible values.

if pos == "none" {
    updated = "ō" + updated;
}

// `language` is also available

if language == "Greek" {
    updated.remove("t");
}

// The tags associated with the word are also exported
if tags.contains("example") {
    updated = updated + "a";
}

//the historical_metadata field is also exported
if metadata.contains("word") {
    updated = updated + "l";
}

// return the value
updated