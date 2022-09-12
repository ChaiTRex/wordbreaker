use fst::raw::Fst;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
#[repr(transparent)]
pub struct Dictionary<D> {
    fst: Fst<D>,
}

impl<'a> Dictionary<Vec<u8>> {
    pub fn from_iter<I>(words: I) -> fst::Result<Self>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut words = words
            .filter(|word| !word.is_empty())
            .map(|word| word.chars().nfd().collect::<String>())
            .collect::<Vec<_>>();
        words.sort_unstable_by(|word1, word2| word1.as_bytes().cmp(word2.as_bytes()));
        words.dedup();

        Fst::from_iter_set(words.into_iter()).map(|fst| Dictionary { fst })
    }
}

impl Dictionary<Vec<u8>> {
    pub fn new<T>(words: &[T]) -> fst::Result<Self>
    where
        T: AsRef<str>,
    {
        Self::from_iter(words.iter().map(|word| word.as_ref()))
    }
}

impl<D> Dictionary<D>
where
    D: AsRef<[u8]>,
{
    pub fn concatenations_for(&self, word: &str) -> Vec<Vec<String>> {
        fn _concatenations_for<D, S>(
            dictionary: &Fst<D>,
            graphemes: &[S],
            prefix: &mut Vec<String>,
            results: &mut Vec<Vec<String>>,
        ) where
            D: AsRef<[u8]>,
            S: AsRef<str>,
        {
            prefix.push(String::new());
            let mut current_node = dictionary.root();

            'outer: for (i, grapheme) in graphemes
                .iter()
                .map(|grapheme| grapheme.as_ref())
                .enumerate()
            {
                prefix.last_mut().unwrap().push_str(grapheme);

                for byte in grapheme.bytes() {
                    let transition_index = match current_node.find_input(byte) {
                        Some(index) => index,
                        None => break 'outer,
                    };
                    current_node = dictionary.node(current_node.transition(transition_index).addr);
                }

                if current_node.is_final() {
                    if i == graphemes.len() - 1 {
                        results.push(prefix.clone());
                    } else {
                        _concatenations_for(dictionary, &graphemes[i + 1..], prefix, results);
                    }
                }
            }

            prefix.pop();
        }

        if word.is_empty() {
            return vec![Vec::new()];
        }

        let word = word.chars().nfd().collect::<String>();
        let graphemes = word.graphemes(true).collect::<Vec<_>>();
        let mut results = Vec::new();

        _concatenations_for(&self.fst, &graphemes, &mut Vec::new(), &mut results);

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
