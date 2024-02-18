fn run(a) {
    let change_next_vowel = false;
    let new_acc = "";
    let match_acc = "";
    print("a: " + a);
    for letter in a {
        
        match_acc = match_acc + letter;
        if match_acc == "šw" {
            change_next_vowel = true;
        }
        
        if letter == "w" {
            continue
        }
        
        if change_next_vowel && (letter == "u" || letter == "i") {
            new_acc = new_acc + "ō";
            change_next_vowel = false;
        } else {
            new_acc = new_acc + letter;
        }
    }
    return new_acc;
}
print("metadata: " + metadata);
run(lemma_array);