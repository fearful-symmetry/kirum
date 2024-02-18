let acc = [];
for letter in lemma_array {
    print("letter: " + letter);
    if len(letter) > 1 && letter[1] == "ʷ" {
        acc.push(letter[0]);
    } else {
        acc.push(letter);
    }
}

let last = acc.pop();
let last = last + "ʷ";
acc.push(last);
acc