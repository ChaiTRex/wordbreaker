use alloc::string::String;
use alloc::vec::Vec;
use fst::raw::Fst;
use unicode_normalization::UnicodeNormalization;

use crate::WordSegmentations;

pub use fst::raw::Error;

/// Stores a dictionary's words so that word segmentation is speedy. Canonicalizes the
/// Unicode to NFD form.
///
/// <code>D</code> is the backing storage for the dictionary, which must implement
/// <code>[AsRef](core::convert::AsRef)&lt;&#91;[u8](core::primitive::u8)&#93;&gt;</code>.
#[derive(Clone)]
#[repr(transparent)]
pub struct Dictionary<D> {
    pub(crate) fst: Fst<D>,
}

impl Dictionary<Vec<u8>> {
    /// Creates a new
    /// <code>[Dictionary](crate::Dictionary)&lt;[Vec](alloc::vec::Vec)&lt;[u8](core::primitive::u8)&gt;&gt;</code>
    /// from its <code>words</code>.
    ///
    /// <b>Note:</b> capitalization is preserved, so the words "Arrow" and "box" will
    /// not be a valid segmentation of "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = ["hello", "just", "ice", "justice"]
    ///     .iter()
    ///     .collect::<Dictionary<_>>();
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
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
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
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
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
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
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
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

    /// Finds all segmentations into <code>[Dictionary](crate::Dictionary)</code> words
    /// of the given <code>input</code> string.
    ///
    /// <b>Note:</b> capitalization is preserved, so the words "Arrow" and "box" will
    /// not be a valid segmentation of "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = Dictionary::new(&["hello", "just", "ice", "justice"]);
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
    /// ```
    #[inline(always)]
    pub fn word_segmentations<'s>(&self, input: &'s str) -> WordSegmentations<'s> {
        WordSegmentations::new(self, input)
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
    /// not be a valid segmentation of "arrowbox".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wordbreaker::Dictionary;
    ///
    /// let dictionary = ["hello", "just", "ice", "justice"]
    ///     .iter()
    ///     .collect::<Dictionary<_>>();
    /// let mut word_segmentations = dictionary
    ///     .word_segmentations("justice")
    ///     .collect::<Vec<_>>();
    ///
    /// word_segmentations.sort_unstable();
    /// assert_eq!(word_segmentations, [vec!["just", "ice"], vec!["justice"]]);
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
