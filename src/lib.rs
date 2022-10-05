/*!
<code>wordbreaker</code> is a Unicode-aware <code>no_std</code> crate (requires
<code>[alloc](alloc)</code>) that rapidly finds all ways of segmenting a given string
into words from a given dictionary.
*/

// TODO: remove
//#![cfg_attr(not(test), no_std)]
#![allow(unused_unsafe)]

#[macro_use]
extern crate alloc;

mod dict;
#[doc(inline)]
pub use dict::{Dictionary, Error};

mod iter;
#[doc(inline)]
pub use iter::WordSegmentations;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abcdef_test() {
        let dictionary = Dictionary::new(&["ab", "abc", "cd", "def", "abcd", "ef", "c"]);
        let word_segmentations = dictionary.word_segmentations("abcdef");

        assert_eq!(
            word_segmentations.clone().collect::<Vec<_>>(),
            [
                vec!["ab", "c", "def"],
                vec!["ab", "cd", "ef"],
                vec!["abc", "def"],
                vec!["abcd", "ef"]
            ]
        );
        assert_eq!(word_segmentations.clone().count(), 4);
        assert_eq!(word_segmentations.clone().last(), Some(vec!["abcd", "ef"]));
        assert_eq!(
            word_segmentations.clone().min(),
            Some(vec!["ab", "c", "def"])
        );
        assert_eq!(word_segmentations.clone().max(), Some(vec!["abcd", "ef"]));
        {
            let mut word_segmentations = word_segmentations.clone();
            assert_eq!(word_segmentations.next(), Some(vec!["ab", "c", "def"]));
            assert_eq!(word_segmentations.next(), Some(vec!["ab", "cd", "ef"]));
            assert_eq!(word_segmentations.next(), Some(vec!["abc", "def"]));
            assert_eq!(word_segmentations.next(), Some(vec!["abcd", "ef"]));
            assert_eq!(word_segmentations.next(), None);
        }
        {
            let mut word_segmentations = word_segmentations.clone();
            assert_eq!(word_segmentations.next_back(), Some(vec!["abcd", "ef"]));
            assert_eq!(word_segmentations.next_back(), Some(vec!["abc", "def"]));
            assert_eq!(word_segmentations.next_back(), Some(vec!["ab", "cd", "ef"]));
            assert_eq!(word_segmentations.next_back(), Some(vec!["ab", "c", "def"]));
            assert_eq!(word_segmentations.next_back(), None);
        }
        assert_eq!(word_segmentations.clone().nth(2), Some(vec!["abc", "def"]));
        assert_eq!(word_segmentations.clone().nth(4), None);
        assert_eq!(
            word_segmentations.clone().nth_back(2),
            Some(vec!["ab", "cd", "ef"])
        );
        assert_eq!(word_segmentations.clone().nth_back(4), None);
    }

    #[test]
    fn count_matches_repeated_next_back_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let word_segmentations =
            dictionary.word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        let count = word_segmentations.clone().count();
        let next_back_count = word_segmentations.rev().map(|_| 1).sum::<usize>();

        assert_eq!(count, next_back_count);
    }

    #[test]
    fn count_matches_repeated_next_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let word_segmentations =
            dictionary.word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        let count = word_segmentations.clone().count();
        let next_count = word_segmentations.map(|_| 1).sum::<usize>();

        assert_eq!(count, next_count);
    }

    #[test]
    fn empty_input_test() {
        let dictionary = Dictionary::new(&["b"]);
        let word_segmentations = dictionary.word_segmentations("");

        assert_eq!(
            word_segmentations.clone().collect::<Vec<_>>(),
            [Vec::<&'static str>::new()]
        );
        assert_eq!(word_segmentations.clone().count(), 1);
        assert_eq!(word_segmentations.clone().last(), Some(vec![]));
        assert_eq!(word_segmentations.clone().min(), Some(vec![]));
        assert_eq!(word_segmentations.clone().max(), Some(vec![]));
        {
            let mut word_segmentations = word_segmentations.clone();
            assert_eq!(word_segmentations.next(), Some(vec![]));
            assert_eq!(word_segmentations.next(), None);
        }
        {
            let mut word_segmentations = word_segmentations.clone();
            assert_eq!(word_segmentations.next_back(), Some(vec![]));
            assert_eq!(word_segmentations.next_back(), None);
        }
        assert_eq!(word_segmentations.clone().nth(0), Some(vec![]));
        assert_eq!(word_segmentations.clone().nth(1), None);
        assert_eq!(word_segmentations.clone().nth_back(0), Some(vec![]));
        assert_eq!(word_segmentations.clone().nth_back(1), None);
    }

    #[test]
    fn first_matches_repeated_next_back_test() {
        let word_segmentations = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>()
            .word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        let first = word_segmentations.clone().next();
        let mut next_back_last = None;
        for item in word_segmentations.rev() {
            next_back_last = Some(item);
        }

        assert_eq!(first, next_back_last);
    }

    #[test]
    fn from_bytes_verified_test() {
        let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
        let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();

        let dictionary = Dictionary::from_bytes_verified(first_dictionary_bytes).unwrap();
        let mut ways_to_segment_into_words =
            dictionary.word_segmentations("justice").collect::<Vec<_>>();

        ways_to_segment_into_words.sort_unstable();
        assert_eq!(
            ways_to_segment_into_words,
            [vec!["just", "ice"], vec!["justice"]]
        );
    }

    #[test]
    fn last_matches_repeated_next_test() {
        let word_segmentations = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>()
            .word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        let last = word_segmentations.clone().last();
        let mut next_last = None;
        for item in word_segmentations {
            next_last = Some(item);
        }

        assert_eq!(last, next_last);
    }

    #[test]
    fn no_matching_word_segmentations_test() {
        let dictionary = Dictionary::new(&["b"]);
        let word_segmentations = dictionary.word_segmentations("a");

        assert!(word_segmentations.clone().collect::<Vec<_>>().is_empty());
        assert_eq!(word_segmentations.clone().count(), 0);
        assert_eq!(word_segmentations.clone().last(), None);
        assert_eq!(word_segmentations.clone().min(), None);
        assert_eq!(word_segmentations.clone().max(), None);
        assert_eq!(word_segmentations.clone().next(), None);
        assert_eq!(word_segmentations.clone().next_back(), None);
        assert_eq!(word_segmentations.clone().nth(0), None);
        assert_eq!(word_segmentations.clone().nth_back(0), None);
    }

    #[test]
    fn nth_back_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let quick_brown_fox_iter =
            dictionary.word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        assert_eq!(
            quick_brown_fox_iter.clone().nth_back(22),
            Some(vec![
                "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"
            ]),
        );

        let mut check_iter = quick_brown_fox_iter.clone();
        for n in 0..quick_brown_fox_iter.clone().count().wrapping_add(100) {
            assert_eq!(
                check_iter.next_back(),
                quick_brown_fox_iter.clone().nth_back(n)
            );
        }
    }

    #[test]
    fn nth_test() {
        let dictionary = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>();
        let quick_brown_fox_iter =
            dictionary.word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        assert_eq!(
            quick_brown_fox_iter.clone().nth(71257),
            Some(vec![
                "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"
            ]),
        );

        let mut check_iter = quick_brown_fox_iter.clone();
        for n in 0..quick_brown_fox_iter.clone().count().wrapping_add(100) {
            assert_eq!(check_iter.next(), quick_brown_fox_iter.clone().nth(n));
        }
    }

    #[test]
    fn size_hint_test() {
        let word_segmentations = include_str!("../american-english-dictionary.txt")
            .lines()
            .collect::<Dictionary<_>>()
            .word_segmentations("thequickbrownfoxjumpsoverthelazydog");

        let (count, option_count) = word_segmentations.size_hint();

        let mut forward_iter = word_segmentations.clone();
        for _ in 0..count {
            assert!(forward_iter.next().is_some());
        }

        let mut reverse_iter = word_segmentations.clone();
        for _ in 0..count {
            assert!(reverse_iter.next_back().is_some());
        }

        assert_eq!(Some(count), option_count);
        assert_eq!(forward_iter.next(), None);
        assert_eq!(reverse_iter.next_back(), None);
    }
}
