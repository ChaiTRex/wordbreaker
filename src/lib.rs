/*!
<code>wordbreaker</code> is a Unicode-aware <code>no_std</code> crate (requires
<code>[alloc](alloc)</code>) that rapidly finds all concatenations of words in a
dictionary that produce a certain input string.
*/

#![no_std]

#[macro_use]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use fst::raw::{Fst, Node};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Stores a dictionary's words so that concatenation finding is speedy. Canonicalizes
/// the Unicode to NFD form.
///
/// <code>D</code> is the backing storage for the dictionary, which must implement
/// <code>[AsRef](core::convert::AsRef)&lt;&#91;[u8](core::primitive::u8)&#93;&gt;</code>.
#[derive(Clone)]
#[repr(transparent)]
pub struct Dictionary<D> {
    fst: Fst<D>,
}

impl Dictionary<Vec<u8>> {
    /// Creates a new
    /// <code>[Dictionary](crate::Dictionary)&lt;[Vec](alloc::vec::Vec)&lt;[u8](core::primitive::u8)&gt;&gt;</code>
    /// from an <code>[Iterator](core::iter::Iterator)</code> over strings.
    ///
    /// Note: capitalization is preserved, so the words "Arrow" and "box" will not
    /// concatenate to "arrowbox".
    pub fn from_iter<I, S>(words: I) -> fst::Result<Self>
    where
        I: Iterator<Item = S>,
        S: AsRef<str>,
    {
        let mut words = words
            .filter_map(|word| {
                let word = word.as_ref();
                if word.is_empty() {
                    None
                } else {
                    Some(word.chars().nfd().collect::<String>())
                }
            })
            .collect::<Vec<_>>();
        words.sort_unstable_by(|word1, word2| word1.as_bytes().cmp(word2.as_bytes()));
        words.dedup();

        Fst::from_iter_set(words.into_iter()).map(|fst| Dictionary { fst })
    }

    /// Creates a new
    /// <code>[Dictionary](crate::Dictionary)&lt;[Vec](alloc::vec::Vec)&lt;[u8](core::primitive::u8)&gt;&gt;</code>
    /// from its <code>words</code>.
    ///
    /// Note: capitalization is preserved, so the words "Arrow" and "box" will not
    /// concatenate to "arrowbox".
    pub fn new<S>(words: &[S]) -> fst::Result<Self>
    where
        S: AsRef<str>,
    {
        Self::from_iter(words.iter())
    }
}

impl<D> Dictionary<D>
where
    D: AsRef<[u8]>,
{
    /// Gets the underlying bytes of a <code>[Dictionary](crate::Dictionary)</code> so
    /// that they can be stored (for example, on disk) and later loaded (for example,
    /// using <code>[include_bytes!](core::include_bytes)</code>) to recreate the
    /// <code>[Dictionary](crate::Dictionary)</code> using
    /// <code>[Dictionary](crate::Dictionary)::[from_bytes](crate::Dictionary::from_bytes)</code>.
    ///
    /// Note: the byte format of the <code>[Dictionary](crate::Dictionary)</code> may
    /// change on major updates of this library, requiring the bytes of a
    /// <code>[Dictionary](crate::Dictionary)</code> to be regenerated in the new format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]).unwrap();
    /// let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();
    ///
    /// let dictionary = Dictionary::from_bytes(first_dictionary_bytes).unwrap();
    /// let mut ways_to_concatenate = dictionary.concatenations_for("justice");
    /// ways_to_concatenate.sort_unstable();
    ///
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.fst.as_bytes()
    }

    /// Creates a new <code>[Dictionary](crate::Dictionary)</code> from the underlying
    /// bytes of a prior <code>[Dictionary](crate::Dictionary)</code>, which can be
    /// produced by
    /// <code>[Dictionary](crate::Dictionary)::[as_bytes](crate::Dictionary::as_bytes)</code>.
    /// This avoids the expense of processing a lot of words to create a
    /// <code>[Dictionary](crate::Dictionary)</code>, as they were already processed
    /// when the prior <code>[Dictionary](crate::Dictionary)</code> was created.
    ///
    /// This can be used in conjuction with loading the bytes from disk (perhaps by
    /// using <code>[include_bytes!](core::include_bytes)</code>).
    ///
    /// Note: the byte format of the <code>[Dictionary](crate::Dictionary)</code> may
    /// change on major updates of this library, requiring the bytes of a
    /// <code>[Dictionary](crate::Dictionary)</code> to be regenerated in the new format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]).unwrap();
    /// let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();
    ///
    /// let dictionary = Dictionary::from_bytes(first_dictionary_bytes).unwrap();
    /// let mut ways_to_concatenate = dictionary.concatenations_for("justice");
    /// ways_to_concatenate.sort_unstable();
    ///
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    pub fn from_bytes(bytes: D) -> fst::Result<Dictionary<D>> {
        Fst::new(bytes).map(|fst| Dictionary { fst })
    }

    /// Finds all concatenations of words in this
    /// <code>[Dictionary](crate::Dictionary)</code> that produce the <code>input</code>
    /// string.
    ///
    /// Note: capitalization is preserved, so the words "Arrow" and "box" will not
    /// concatenate to "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]).unwrap();
    /// let mut ways_to_concatenate = dictionary.concatenations_for("justice");
    /// ways_to_concatenate.sort_unstable();
    ///
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    pub fn concatenations_for(&self, input: &str) -> Vec<Vec<String>> {
        if input.is_empty() {
            return vec![Vec::new()];
        }

        let dictionary = &self.fst;
        let input = input.chars().nfd().collect::<String>();
        let graphemes = input.graphemes(true);
        let last_grapheme_index = graphemes.clone().count().wrapping_sub(1);
        let mut prefix = vec![String::new()];
        let mut results = Vec::new();

        struct StackFrame<'s, 'n, I>
        where
            I: Clone + Iterator<Item = (usize, &'s str)>,
        {
            graphemes: I,
            current_node: Node<'n>,
        }

        let mut stack = vec![StackFrame {
            graphemes: graphemes.enumerate(),
            current_node: dictionary.root(),
        }];
        'outer: loop {
            match stack.pop() {
                Some(mut stack_frame) => {
                    if let Some((i, grapheme)) = stack_frame.graphemes.next() {
                        prefix.last_mut().unwrap().push_str(grapheme);

                        for byte in grapheme.bytes() {
                            let transition_index = match stack_frame.current_node.find_input(byte) {
                                Some(index) => index,
                                None => {
                                    prefix.pop();
                                    continue 'outer;
                                }
                            };
                            stack_frame.current_node = dictionary
                                .node(stack_frame.current_node.transition(transition_index).addr);
                        }

                        if stack_frame.current_node.is_final() {
                            if i == last_grapheme_index {
                                results.push(prefix.clone());
                                prefix.pop();
                            } else {
                                prefix.push(String::new());

                                let graphemes_clone = stack_frame.graphemes.clone();
                                stack.push(stack_frame);
                                stack.push(StackFrame {
                                    graphemes: graphemes_clone,
                                    current_node: dictionary.root(),
                                });
                            }
                        } else {
                            stack.push(stack_frame);
                        }
                    } else {
                        prefix.pop();
                    }
                }
                None => break,
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let dictionary = Dictionary::new(&["b"]).unwrap();
        let ways_to_concatenate = dictionary.concatenations_for("a");

        assert!(ways_to_concatenate.is_empty());
    }

    #[test]
    fn test_2() {
        let dictionary = Dictionary::new(&["b"]).unwrap();
        let ways_to_concatenate = dictionary.concatenations_for("");

        assert!(ways_to_concatenate.len() == 1);
        assert!(ways_to_concatenate[0].is_empty());
    }

    #[test]
    fn test_3() {
        let dictionary = Dictionary::new(&["ab", "abc", "cd", "def", "abcd", "ef", "c"]).unwrap();
        let mut ways_to_concatenate = dictionary.concatenations_for("abcdef");
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
}
