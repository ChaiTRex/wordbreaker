`wordbreaker` is a Unicode-aware `no_std` crate (requires [`alloc`](alloc)) that rapidly
finds all sequences of dictionary words that concatenate to a given string.

# Example

```rust
use wordbreaker::Dictionary;

let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
let mut ways_to_concatenate = dictionary
    .concatenations_for("justice")
    .collect::<Vec<_>>();

ways_to_concatenate.sort_unstable();
assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
```
