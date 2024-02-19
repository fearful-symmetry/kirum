let updated = lemma_string;
for t in tags{
    updated = updated + "-" + t;
}

for v in metadata.values() {
    updated =  updated + "&map:" + v
}

updated