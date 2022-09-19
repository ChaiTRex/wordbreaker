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
        let concatenations = dictionary.concatenations_for("abcdef");

        assert_eq!(
            concatenations.clone().collect::<Vec<_>>(),
            [
                vec!["ab", "c", "def"],
                vec!["ab", "cd", "ef"],
                vec!["abc", "def"],
                vec!["abcd", "ef"]
            ]
        );
        assert_eq!(concatenations.clone().count(), 4);
        assert_eq!(concatenations.clone().last(), Some(vec!["abcd", "ef"]));
        assert_eq!(concatenations.clone().min(), Some(vec!["ab", "c", "def"]));
        assert_eq!(concatenations.clone().max(), Some(vec!["abcd", "ef"]));
        {
            let mut concatenations = concatenations.clone();
            assert_eq!(concatenations.next(), Some(vec!["ab", "c", "def"]));
            assert_eq!(concatenations.next(), Some(vec!["ab", "cd", "ef"]));
            assert_eq!(concatenations.next(), Some(vec!["abc", "def"]));
            assert_eq!(concatenations.next(), Some(vec!["abcd", "ef"]));
            assert_eq!(concatenations.next(), None);
        }
        assert_eq!(concatenations.clone().nth(2), Some(vec!["abc", "def"]));
        assert_eq!(concatenations.clone().nth(4), None);
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
        let concatenations = dictionary.concatenations_for("");

        assert_eq!(
            concatenations.clone().collect::<Vec<_>>(),
            [Vec::<&'static str>::new()]
        );
        assert_eq!(concatenations.clone().count(), 1);
        assert_eq!(concatenations.clone().last(), Some(vec![]));
        assert_eq!(concatenations.clone().min(), Some(vec![]));
        assert_eq!(concatenations.clone().max(), Some(vec![]));
        {
            let mut concatenations = concatenations.clone();
            assert_eq!(concatenations.next(), Some(vec![]));
            assert_eq!(concatenations.next(), None);
        }
        assert_eq!(concatenations.clone().nth(0), Some(vec![]));
        assert_eq!(concatenations.clone().nth(1), None);
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
    fn last_matches_repeated_next_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let mut concatenations =
            dictionary.concatenations_for("thequickbrownfoxjumpsoverthelazydog");

        let last = concatenations.clone().last();
        let mut next_last = None;
        loop {
            let next = concatenations.next();
            if next.is_some() {
                next_last = next;
            } else {
                break;
            }
        }

        assert_eq!(last, next_last);
    }

    #[test]
    fn no_matching_concatenations_test() {
        let dictionary = Dictionary::new(&["b"]);
        let concatenations = dictionary.concatenations_for("a");

        assert!(concatenations.clone().collect::<Vec<_>>().is_empty());
        assert_eq!(concatenations.clone().count(), 0);
        assert_eq!(concatenations.clone().last(), None);
        assert_eq!(concatenations.clone().min(), None);
        assert_eq!(concatenations.clone().max(), None);
        assert_eq!(concatenations.clone().next(), None);
        assert_eq!(concatenations.clone().nth(0), None);
    }

    #[test]
    fn nth_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let expected_concatenation = dictionary
            .concatenations_for("thequickbrownfoxjumpsoverthelazydog")
            .nth(71257);

        assert_eq!(
            expected_concatenation,
            Some(vec![
                "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"
            ]),
        );
    }
}
