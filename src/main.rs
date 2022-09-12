use fst::raw::{Fst, Node};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

fn create_dictionary_fst<T: AsRef<str>>(words: &[T]) -> fst::Result<Fst<Vec<u8>>> {
    let mut words = words
        .iter()
        .map(|word| word.as_ref())
        .filter(|word| !word.is_empty())
        .map(|word| word.chars().nfd().collect::<String>())
        .collect::<Vec<_>>();
    words.sort_unstable_by(|word1, word2| word1.as_bytes().cmp(word2.as_bytes()));
    words.dedup();

    Fst::from_iter_set(words.iter())
}

fn break_into_words<D: AsRef<[u8]>>(word: &str, dictionary: &Fst<D>) -> Vec<Vec<String>> {
    fn _break_into_words<D: AsRef<[u8]>, S: AsRef<str>>(
        results: &mut Vec<Vec<String>>,
        prefix: &mut Vec<String>,
        graphemes: &[S],
        dictionary: &Fst<D>,
        root: Node<'_>,
    ) {
        prefix.push(String::new());
        let mut current_node = root;

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
                    _break_into_words(results, prefix, &graphemes[i + 1..], dictionary, root);
                }
            }
        }

        prefix.pop();
    }

    if word.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut prefix = Vec::new();
    let word = word.chars().nfd().collect::<String>();
    let graphemes = word.graphemes(true).collect::<Vec<_>>();
    let root = dictionary.root();

    _break_into_words(&mut results, &mut prefix, &graphemes, dictionary, root);

    results
}

fn main() {
    let dictionary =
        create_dictionary_fst(&["ab", "abc", "cd", "def", "abcd", "ef", "c", ""]).unwrap();

    dbg!(break_into_words("abcdef", &dictionary));
}
