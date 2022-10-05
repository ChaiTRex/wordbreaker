`wordbreaker` is a Unicode-aware `no_std` crate (requires [`alloc`](alloc)) that rapidly
finds all ways of segmenting a given string into words from a given dictionary.

# Example

```rust
use wordbreaker::Dictionary;

let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
let mut word_segmentations = dictionary
    .word_segmentations("justice")
    .collect::<Vec<_>>();
word_segmentations.sort_unstable();

assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
```
