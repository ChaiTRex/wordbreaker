`wordbreaker` is a `no_std` crate (requires
[`alloc`](https://doc.rust-lang.org/alloc/)) that rapidly finds all
concatenations of words in a dictionary that produce a certain input string.

# Example

```rust
use wordbreaker::Dictionary;

let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]).unwrap();
let mut ways_to_concatenate = dictionary.concatenations_for("justice");
ways_to_concatenate.sort_unstable();

assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
```
