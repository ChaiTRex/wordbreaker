/*!
<code>wordbreaker</code> is a Unicode-aware <code>no_std</code> crate (requires
<code>[alloc](alloc)</code>) that rapidly finds all sequences of dictionary words that
concatenate to a given string.
*/

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;
use fst::raw::{Fst, Node};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

#[doc(inline)]
pub use fst::raw::Error;

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
    /// from its <code>words</code>.
    ///
    /// <b>Note:</b> capitalization is preserved, so the words "Arrow" and "box" will
    /// not concatenate to "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = ["hello", "just", "ice", "justice"]
    ///     .into_iter()
    ///     .collect::<Dictionary<_>>();
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    #[inline]
    pub fn new<S>(words: &[S]) -> Self
    where
        S: AsRef<str>,
    {
        words.iter().collect()
    }
}

impl<D> Dictionary<D>
where
    D: AsRef<[u8]>,
{
    /// Creates a new <code>[Dictionary](crate::Dictionary)</code> from the underlying
    /// bytes of a prior <code>[Dictionary](crate::Dictionary)</code>.
    ///
    /// These bytes can be produced by using the
    /// <code>[Dictionary](crate::Dictionary)::[as_bytes](crate::Dictionary::as_bytes)</code>.
    /// method on the prior <code>[Dictionary](crate::Dictionary)</code>. This avoids
    /// the expense of processing a lot of words to create a
    /// <code>[Dictionary](crate::Dictionary)</code>, as they were already processed
    /// when the prior <code>[Dictionary](crate::Dictionary)</code> was created.
    ///
    /// This can be used in conjuction with loading the bytes from disk (perhaps by
    /// using <code>[include_bytes!](core::include_bytes)</code>).
    ///
    /// <b>Note:</b> the byte format of the <code>[Dictionary](crate::Dictionary)</code>
    /// may change on major updates of this library, requiring the bytes of a
    /// <code>[Dictionary](crate::Dictionary)</code> to be regenerated in the new format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
    /// let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();
    ///
    /// let dictionary = Dictionary::from_bytes(first_dictionary_bytes).unwrap();
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        self.fst.as_bytes()
    }

    /// Creates a new <code>[Dictionary](crate::Dictionary)</code> from the underlying
    /// bytes of a prior <code>[Dictionary](crate::Dictionary)</code>, <b>without</b>
    /// verifying that the checksum is correct.
    ///
    /// These bytes can be produced by using the
    /// <code>[Dictionary](crate::Dictionary)::[as_bytes](crate::Dictionary::as_bytes)</code>.
    /// method on the prior <code>[Dictionary](crate::Dictionary)</code>. This avoids
    /// the expense of processing a lot of words to create a
    /// <code>[Dictionary](crate::Dictionary)</code>, as they were already processed
    /// when the prior <code>[Dictionary](crate::Dictionary)</code> was created.
    ///
    /// This can be used in conjuction with loading the bytes from disk (perhaps by
    /// using <code>[include_bytes!](core::include_bytes)</code>).
    ///
    /// <b>Note:</b> the byte format of the <code>[Dictionary](crate::Dictionary)</code>
    /// may change on major updates of this library, requiring the bytes of a
    /// <code>[Dictionary](crate::Dictionary)</code> to be regenerated in the new format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
    /// let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();
    ///
    /// let dictionary = Dictionary::from_bytes(first_dictionary_bytes).unwrap();
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    pub fn from_bytes(bytes: D) -> Result<Dictionary<D>, Error> {
        match Fst::new(bytes) {
            Ok(fst) => Ok(Dictionary { fst }),
            Err(fst::Error::Fst(e)) => Err(e),
            Err(_) => unreachable!("When loading a `Dictionary` from bytes, got an error unrelated to underlying `Fst`"),
        }
    }

    /// Creates a new <code>[Dictionary](crate::Dictionary)</code> from the underlying
    /// bytes of a prior <code>[Dictionary](crate::Dictionary)</code>, verifying that the
    /// checksum is correct.
    ///
    /// These bytes can be produced by using the
    /// <code>[Dictionary](crate::Dictionary)::[as_bytes](crate::Dictionary::as_bytes)</code>.
    /// method on the prior <code>[Dictionary](crate::Dictionary)</code>. This avoids
    /// the expense of processing a lot of words to create a
    /// <code>[Dictionary](crate::Dictionary)</code>, as they were already processed
    /// when the prior <code>[Dictionary](crate::Dictionary)</code> was created.
    ///
    /// This can be used in conjuction with loading the bytes from disk (perhaps by
    /// using <code>[include_bytes!](core::include_bytes)</code>).
    ///
    /// <b>Note:</b> the byte format of the <code>[Dictionary](crate::Dictionary)</code>
    /// may change on major updates of this library, requiring the bytes of a
    /// <code>[Dictionary](crate::Dictionary)</code> to be regenerated in the new format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let first_dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
    /// let first_dictionary_bytes = first_dictionary.as_bytes().to_vec();
    ///
    /// let dictionary = Dictionary::from_bytes_verified(first_dictionary_bytes).unwrap();
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    pub fn from_bytes_verified(bytes: D) -> Result<Dictionary<D>, Error> {
        match Fst::new(bytes).and_then(|fst| {
            fst.verify()?;
            Ok(fst)
        }) {
            Ok(fst) => Ok(Dictionary { fst }),
            Err(fst::Error::Fst(e)) => Err(e),
            Err(_) => unreachable!("When loading a `Dictionary` from bytes, got an error unrelated to underlying `Fst`"),
        }
    }

    /// Finds all concatenations of words in this
    /// <code>[Dictionary](crate::Dictionary)</code> that produce the <code>input</code>
    /// string.
    ///
    /// <b>Note:</b> capitalization is preserved, so the words "Arrow" and "box" will
    /// not concatenate to "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    #[inline(always)]
    pub fn concatenations_for<'d, 's>(&'d self, input: &'s str) -> Concatenations<'d, 's, D> {
        Concatenations::new(self, input)
    }
}

impl<S> core::iter::FromIterator<S> for Dictionary<Vec<u8>>
where
    S: AsRef<str>,
{
    /// Creates a new
    /// <code>[Dictionary](crate::Dictionary)&lt;[Vec](alloc::vec::Vec)&lt;[u8](core::primitive::u8)&gt;&gt;</code>
    /// from an <code>[Iterator](core::iter::Iterator)</code> over strings.
    ///
    /// <b>Note:</b> capitalization is preserved, so the words "Arrow" and "box" will
    /// not concatenate to "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = ["hello", "just", "ice", "justice"]
    ///     .into_iter()
    ///     .collect::<Dictionary<_>>();
    /// let mut ways_to_concatenate = dictionary
    ///     .concatenations_for("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// ways_to_concatenate.sort_unstable();
    /// assert_eq!(ways_to_concatenate, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    fn from_iter<I>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
    {
        let mut words = words
            .into_iter()
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

        Dictionary {
            fst: Fst::from_iter_set(words.into_iter()).unwrap(),
        }
    }
}

/// The <code>[Iterator](core::iter::Iterator)</code> that
/// <code>[Dictionary](crate::Dictionary)::[concatenations_for](crate::Dictionary::concatenations_for)</code>
/// produces.
pub struct Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    dictionary: &'d Fst<D>,
    input: &'s str,
    input_nfd: String,
    grapheme_bounds: Vec<GraphemeBounds>,
    stack: Vec<StackFrame<'d>>,
    prefix: Vec<Range<usize>>,
}

impl<'d, 's, D> Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    fn new(dictionary: &'d Dictionary<D>, input: &'s str) -> Self {
        let dictionary = &dictionary.fst;
        let input_nfd = input.nfd().collect::<String>();
        let input_grapheme_ends =
            input
                .graphemes(true)
                .map(|grapheme| grapheme.len())
                .scan(0_usize, |prev, this| {
                    *prev = (*prev).wrapping_add(this);
                    Some(*prev)
                });
        let input_nfd_grapheme_ends = input_nfd
            .graphemes(true)
            .map(|grapheme| grapheme.len())
            .scan(0_usize, |prev, this| {
                *prev = (*prev).wrapping_add(this);
                Some(*prev)
            });
        let grapheme_bounds = core::iter::once({
            GraphemeBounds {
                input_index: 0,
                input_nfd_index: 0,
            }
        })
        .chain({
            input_grapheme_ends.zip(input_nfd_grapheme_ends).map(
                |(input_index, input_nfd_index)| GraphemeBounds {
                    input_index,
                    input_nfd_index,
                },
            )
        })
        .collect::<Vec<_>>();

        Self {
            dictionary,
            input,
            input_nfd,
            grapheme_bounds,
            stack: vec![StackFrame {
                grapheme_end_index: 1,
                current_node: dictionary.root(),
            }],
            prefix: vec![0..0],
        }
    }
}

impl<'d, 's, D> Iterator for Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    type Item = Vec<&'s str>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: while let Some(mut stack_frame) = self.stack.pop() {
            if stack_frame.grapheme_end_index < self.grapheme_bounds.len() {
                let input_nfd_grapheme_start = self.grapheme_bounds
                    [stack_frame.grapheme_end_index.wrapping_sub(1)]
                .input_nfd_index;
                let GraphemeBounds {
                    input_index: input_grapheme_end,
                    input_nfd_index: input_nfd_grapheme_end,
                } = self.grapheme_bounds[stack_frame.grapheme_end_index];
                stack_frame.grapheme_end_index = stack_frame.grapheme_end_index.wrapping_add(1);

                self.prefix.last_mut().unwrap().end = input_grapheme_end;

                for byte in self.input_nfd[input_nfd_grapheme_start..input_nfd_grapheme_end].bytes()
                {
                    let transition_index = match stack_frame.current_node.find_input(byte) {
                        Some(index) => index,
                        None => {
                            self.prefix.pop();
                            continue 'outer;
                        }
                    };
                    stack_frame.current_node = self
                        .dictionary
                        .node(stack_frame.current_node.transition(transition_index).addr);
                }

                if stack_frame.current_node.is_final() {
                    if input_grapheme_end == self.input.len() {
                        if self.stack.is_empty() {
                            // We're done, so drop all owned values now, replacing them
                            // with unallocated values, in case the programmer keeps
                            // this iterator around.

                            self.input_nfd = String::new();
                            self.grapheme_bounds = Vec::new();
                            self.stack = Vec::new();
                            return Some({
                                core::mem::take(&mut self.prefix)
                                    .into_iter()
                                    .map(|range| &self.input[range])
                                    .collect::<Vec<_>>()
                            });
                        } else {
                            let next = self
                                .prefix
                                .iter()
                                .cloned()
                                .map(|range| &self.input[range])
                                .collect::<Vec<_>>();
                            self.prefix.pop();
                            return Some(next);
                        }
                    } else {
                        self.prefix.push(input_grapheme_end..input_grapheme_end);

                        let grapheme_end_index = stack_frame.grapheme_end_index;
                        self.stack.push(stack_frame);
                        self.stack.push(StackFrame {
                            grapheme_end_index,
                            current_node: self.dictionary.root(),
                        });
                    }
                } else {
                    self.stack.push(stack_frame);
                }
            } else {
                if self.input.is_empty() {
                    // We're done, so drop all owned values now, replacing them with
                    // unallocated values, in case the programmer keeps this iterator
                    // around.

                    self.input_nfd = String::new();
                    self.grapheme_bounds = Vec::new();
                    self.stack = Vec::new();
                    self.prefix = Vec::new();
                    return Some(Vec::new());
                }
                self.prefix.pop();
            }
        }

        None
    }
}

struct GraphemeBounds {
    input_index: usize,
    input_nfd_index: usize,
}

struct StackFrame<'d> {
    grapheme_end_index: usize,
    current_node: Node<'d>,
}

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
