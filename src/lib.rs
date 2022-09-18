/*!
<code>wordbreaker</code> is a Unicode-aware <code>no_std</code> crate (requires
<code>[alloc](alloc)</code>) that rapidly finds all sequences of dictionary words that
concatenate to a given string.
*/

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

mod dict;
#[doc(inline)]
pub use dict::{Dictionary, Error};

mod iter;
#[doc(inline)]
pub use iter::Concatenations;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abcdef_test() {
        let dictionary = Dictionary::new(&["ab", "abc", "cd", "def", "abcd", "ef", "c"]);
        let mut ways_to_concatenate = dictionary.concatenations_for("abcdef").collect::<Vec<_>>();

        ways_to_concatenate.sort_unstable();
        assert_eq!(
            ways_to_concatenate,
            [
                vec!["ab", "c", "def"],
                vec!["ab", "cd", "ef"],
                vec!["abc", "def"],
                vec!["abcd", "ef"]
            ]
        );
    }

    #[test]
    fn count_matches_repeated_next_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let concatenations = dictionary.concatenations_for("thequickbrownfoxjumpsoverthelazydog");

        let count = concatenations.clone().count();
        let next_count = concatenations.map(|_| 1).sum::<usize>();

        assert_eq!(count, next_count);
    }

    #[test]
    fn empty_input_test() {
        let dictionary = Dictionary::new(&["b"]);
        let ways_to_concatenate = dictionary.concatenations_for("").collect::<Vec<_>>();

        assert!(ways_to_concatenate.len() == 1);
        assert!(ways_to_concatenate[0].is_empty());
    }

    #[test]
    fn from_bytes_verified_test() {
        let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
        let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();

        let dictionary = Dictionary::from_bytes_verified(first_dictionary_bytes).unwrap();
        let mut ways_to_concatenate = dictionary.concatenations_for("justice").collect::<Vec<_>>();

        ways_to_concatenate.sort_unstable();
        assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    }

    #[test]
    fn no_matching_concatenations_test() {
        let dictionary = Dictionary::new(&["b"]);
        let ways_to_concatenate = dictionary.concatenations_for("a").collect::<Vec<_>>();

        assert!(ways_to_concatenate.is_empty());
    }
}
