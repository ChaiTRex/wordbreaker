use alloc::string::String;
use alloc::vec::Vec;
use fst::raw::{Fst, Node};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::Dictionary;

/// The <code>[Iterator](core::iter::Iterator)</code> that
/// <code>[Dictionary](crate::Dictionary)::[concatenations_for](crate::Dictionary::concatenations_for)</code>
/// produces.
#[derive(Clone)]
pub struct Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    dictionary: &'d Fst<D>,
    input: &'s str,
    input_nfd: String,
    grapheme_bounds: Vec<GraphemeBounds>,
    stack: Vec<StackFrame<'d>>,
    input_prefix_indexes: Vec<usize>,
}

impl<'d, 's, D> Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    pub(crate) fn new(dictionary: &'d Dictionary<D>, input: &'s str) -> Self {
        let dictionary = &dictionary.fst;

        if input.is_empty() {
            Self {
                dictionary,
                input,
                input_nfd: String::new(),
                grapheme_bounds: Vec::new(),
                stack: vec![StackFrame {
                    grapheme_index: 0,
                    current_node: dictionary.root(),
                }],
                input_prefix_indexes: Vec::new(),
            }
        } else {
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
            let grapheme_bounds = input_grapheme_ends
                .zip(input_nfd_grapheme_ends)
                .map(|(input_index, input_nfd_index)| GraphemeBounds {
                    input_index,
                    input_nfd_index,
                })
                .collect::<Vec<_>>();

            Self {
                dictionary,
                input,
                input_nfd,
                grapheme_bounds,
                stack: vec![StackFrame {
                    grapheme_index: 0,
                    current_node: dictionary.root(),
                }],
                input_prefix_indexes: vec![0],
            }
        }
    }
}

impl<'d, 's, D> Iterator for Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    type Item = Vec<&'s str>;

    /*
    /* Nightly
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        // TODO: When this is stabilized, move `.nth(n)` code here and base `.nth()` on
        // this method.
    }
    */
    */

    fn count(mut self) -> usize {
        let mut result = 0_usize;

        // While there are still available stack frames:
        'outer: while let Some(mut stack_frame) = self.stack.pop() {
            // If we're at the end of the input:
            if stack_frame.grapheme_index == self.grapheme_bounds.len() {
                // If the input itself was empty:
                if self.input.is_empty() {
                    // Produce 1 for the empty input's unproduced empty Vec.
                    return 1;
                }
                // If the input itself wasn't empty, backtrack implicitly.
                // If we're not at the end of the input:
            } else {
                // Get the current NFD grapheme's start and end.
                let input_nfd_grapheme_start_index = stack_frame
                    .grapheme_index
                    .checked_sub(1)
                    .map(|i| unsafe { self.grapheme_bounds.get_unchecked(i) }.input_nfd_index)
                    .unwrap_or(0);
                let input_nfd_grapheme_end_index = unsafe {
                    self.grapheme_bounds
                        .get_unchecked(stack_frame.grapheme_index)
                }
                .input_nfd_index;

                // For each byte in the current NFD grapheme:
                for byte in self.input_nfd
                    [input_nfd_grapheme_start_index..input_nfd_grapheme_end_index]
                    .bytes()
                {
                    // If there are words in the Dictionary that also have this byte of
                    // the NFD grapheme:
                    if let Some(transition_index) = stack_frame.current_node.find_input(byte) {
                        // Follow the path that includes this byte of the NFD grapheme
                        // as well.
                        stack_frame.current_node = self
                            .dictionary
                            .node(stack_frame.current_node.transition(transition_index).addr);
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme:
                    } else {
                        // Backtrack.
                        continue 'outer;
                    }
                }

                // If, after the current NFD grapheme is followed in the Dictionary, no
                // words end here:
                if !stack_frame.current_node.is_final() {
                    // Continue with the next NFD grapheme.
                    stack_frame.grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    self.stack.push(stack_frame);
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here and there are NFD graphemes left:
                } else if input_nfd_grapheme_end_index < self.input_nfd.len() {
                    // Recurse.
                    let grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    stack_frame.grapheme_index = grapheme_index;
                    self.stack.push(stack_frame);
                    self.stack.push(StackFrame {
                        grapheme_index,
                        current_node: self.dictionary.root(),
                    });
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, and there are no NFD graphemes left:
                } else {
                    #[cfg(debug_assertions)]
                    {
                        result = result.checked_add(1).expect(
                            "The intended result from `Concatenations::count` has overflowed",
                        );
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        result = result.wrapping_add(1);
                    }

                    // Backtrack.
                }
            }
        }

        result
    }

    fn last(mut self) -> Option<Self::Item> {
        let mut last = Vec::new();

        // While there are still available stack frames:
        'outer: while let Some(mut stack_frame) = self.stack.pop() {
            // If we're at the end of the input:
            if stack_frame.grapheme_index == self.grapheme_bounds.len() {
                // If the input itself wasn't empty:
                if !self.input.is_empty() {
                    // Backtrack.
                    self.input_prefix_indexes.pop();
                // If the input itself was empty:
                } else {
                    // We're done, so we deallocate the stack (the only thing allocated
                    // when the input is empty) in case the programmer keeps this
                    // iterator around.
                    self.stack = Vec::new();

                    // Produce an empty Vec for an empty input.
                    return Some(Vec::new());
                }
            // If we're not at the end of the input:
            } else {
                // Get the current input grapheme's end and the current NFD grapheme's
                // start and end.
                let input_nfd_grapheme_start_index = stack_frame
                    .grapheme_index
                    .checked_sub(1)
                    .map(|i| unsafe { self.grapheme_bounds.get_unchecked(i) }.input_nfd_index)
                    .unwrap_or(0);
                let GraphemeBounds {
                    input_index: input_grapheme_end_index,
                    input_nfd_index: input_nfd_grapheme_end_index,
                } = *unsafe {
                    self.grapheme_bounds
                        .get_unchecked(stack_frame.grapheme_index)
                };

                // For each byte in the current NFD grapheme:
                for byte in self.input_nfd
                    [input_nfd_grapheme_start_index..input_nfd_grapheme_end_index]
                    .bytes()
                {
                    // If there are words in the Dictionary that also have this byte of
                    // the NFD grapheme:
                    if let Some(transition_index) = stack_frame.current_node.find_input(byte) {
                        // Follow the path that includes this byte of the NFD grapheme
                        // as well.
                        stack_frame.current_node = self
                            .dictionary
                            .node(stack_frame.current_node.transition(transition_index).addr);
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is more backtracking left to be
                    // done:
                    } else if !self.stack.is_empty() {
                        // Backtrack.
                        self.input_prefix_indexes.pop();
                        continue 'outer;
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is no backtracking left to be done:
                    } else {
                        // We're done, so we deallocate everything allocated in case
                        // the programmer keeps this iterator around.
                        self.input_nfd = String::new();
                        self.grapheme_bounds = Vec::new();
                        self.stack = Vec::new();
                        self.input_prefix_indexes = Vec::new();

                        return if last.is_empty() {
                            None
                        } else {
                            let input_prefix_indexes_iter = last.iter().copied();
                            return Some(
                                core::iter::once(0)
                                    .chain(input_prefix_indexes_iter.clone())
                                    .zip(input_prefix_indexes_iter)
                                    .map(|(start, end)| &self.input[start..end])
                                    .collect::<Vec<_>>(),
                            );
                        };
                    }
                }

                // Adjust the end of this part of the prefix to the end of the current
                // NFD grapheme.
                *self.input_prefix_indexes.last_mut().unwrap() = input_grapheme_end_index;

                // If, after the current NFD grapheme is followed in the Dictionary, no
                // words end here:
                if !stack_frame.current_node.is_final() {
                    // Continue with the next NFD grapheme.
                    stack_frame.grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    self.stack.push(stack_frame);
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here and there are NFD graphemes left:
                } else if input_nfd_grapheme_end_index < self.input_nfd.len() {
                    // Recurse.
                    self.input_prefix_indexes.push(input_grapheme_end_index);

                    let grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    stack_frame.grapheme_index = grapheme_index;
                    self.stack.push(stack_frame);
                    self.stack.push(StackFrame {
                        grapheme_index,
                        current_node: self.dictionary.root(),
                    });
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, there are no NFD graphemes left, and there is more
                // backtracking to be done:
                } else if !self.stack.is_empty() {
                    // Update last.
                    last.clear();
                    last.extend(self.input_prefix_indexes.iter().copied());

                    // Backtrack.
                    self.input_prefix_indexes.pop();
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, there are no NFD graphemes left, and there's no more
                // backtracking to be done:
                } else {
                    let input_prefix_indexes_iter = self.input_prefix_indexes.iter().copied();
                    let result = core::iter::once(0)
                        .chain(input_prefix_indexes_iter.clone())
                        .zip(input_prefix_indexes_iter)
                        .map(|(start, end)| &self.input[start..end])
                        .collect::<Vec<_>>();

                    // We're done, so we deallocate everything allocated in case
                    // the programmer keeps this iterator around.
                    self.input_nfd = String::new();
                    self.grapheme_bounds = Vec::new();
                    self.stack = Vec::new();
                    self.input_prefix_indexes = Vec::new();

                    return Some(result);
                }
            }
        }

        // The work has all been done, so there are no more concatenations to produce.
        None
    }

    #[inline(always)]
    fn max(self) -> Option<Self::Item> {
        // The iterator is sorted, so the maximum remaining element will be the last one.
        self.last()
    }

    #[inline(always)]
    fn min(mut self) -> Option<Self::Item> {
        // The iterator is sorted, so the minimum remaining element will be the next one.
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        // While there are still available stack frames:
        'outer: while let Some(mut stack_frame) = self.stack.pop() {
            // If we're at the end of the input:
            if stack_frame.grapheme_index == self.grapheme_bounds.len() {
                // If the input itself wasn't empty:
                if !self.input.is_empty() {
                    // Backtrack.
                    self.input_prefix_indexes.pop();
                // If the input itself was empty:
                } else {
                    // We're done, so we deallocate the stack (the only thing allocated
                    // when the input is empty) in case the programmer keeps this
                    // iterator around.
                    self.stack = Vec::new();

                    // Produce an empty Vec for an empty input.
                    return Some(Vec::new());
                }
            // If we're not at the end of the input:
            } else {
                // Get the current input grapheme's end and the current NFD grapheme's
                // start and end.
                let input_nfd_grapheme_start_index = stack_frame
                    .grapheme_index
                    .checked_sub(1)
                    .map(|i| unsafe { self.grapheme_bounds.get_unchecked(i) }.input_nfd_index)
                    .unwrap_or(0);
                let GraphemeBounds {
                    input_index: input_grapheme_end_index,
                    input_nfd_index: input_nfd_grapheme_end_index,
                } = *unsafe {
                    self.grapheme_bounds
                        .get_unchecked(stack_frame.grapheme_index)
                };

                // For each byte in the current NFD grapheme:
                for byte in self.input_nfd
                    [input_nfd_grapheme_start_index..input_nfd_grapheme_end_index]
                    .bytes()
                {
                    // If there are words in the Dictionary that also have this byte of
                    // the NFD grapheme:
                    if let Some(transition_index) = stack_frame.current_node.find_input(byte) {
                        // Follow the path that includes this byte of the NFD grapheme
                        // as well.
                        stack_frame.current_node = self
                            .dictionary
                            .node(stack_frame.current_node.transition(transition_index).addr);
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is more backtracking left to be
                    // done:
                    } else if !self.stack.is_empty() {
                        // Backtrack.
                        self.input_prefix_indexes.pop();
                        continue 'outer;
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is no backtracking left to be done:
                    } else {
                        // We're done, so we deallocate everything allocated in case
                        // the programmer keeps this iterator around.
                        self.input_nfd = String::new();
                        self.grapheme_bounds = Vec::new();
                        self.stack = Vec::new();
                        self.input_prefix_indexes = Vec::new();
                        return None;
                    }
                }

                // Adjust the end of this part of the prefix to the end of the current
                // NFD grapheme.
                *self.input_prefix_indexes.last_mut().unwrap() = input_grapheme_end_index;

                // If, after the current NFD grapheme is followed in the Dictionary, no
                // words end here:
                if !stack_frame.current_node.is_final() {
                    // Continue with the next NFD grapheme.
                    stack_frame.grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    self.stack.push(stack_frame);
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here and there are NFD graphemes left:
                } else if input_nfd_grapheme_end_index < self.input_nfd.len() {
                    // Recurse.
                    self.input_prefix_indexes.push(input_grapheme_end_index);

                    let grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    stack_frame.grapheme_index = grapheme_index;
                    self.stack.push(stack_frame);
                    self.stack.push(StackFrame {
                        grapheme_index,
                        current_node: self.dictionary.root(),
                    });
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here and there are no NFD graphemes left:
                } else {
                    let input_prefix_indexes_iter = self.input_prefix_indexes.iter().copied();
                    let result = core::iter::once(0)
                        .chain(input_prefix_indexes_iter.clone())
                        .zip(input_prefix_indexes_iter)
                        .map(|(start, end)| &self.input[start..end])
                        .collect::<Vec<_>>();

                    // If there is some backtracking left to be done:
                    if !self.stack.is_empty() {
                        // Backtrack.
                        self.input_prefix_indexes.pop();
                    // If there's no remaining backtracking left to be done:
                    } else {
                        // We're done, so we deallocate everything allocated in case
                        // the programmer keeps this iterator around.
                        self.input_nfd = String::new();
                        self.grapheme_bounds = Vec::new();
                        self.stack = Vec::new();
                        self.input_prefix_indexes = Vec::new();
                    }

                    return Some(result);
                }
            }
        }

        // The work has all been done, so there are no more concatenations to produce.
        None
    }

    fn nth(&mut self, mut n: usize) -> Option<Self::Item> {
        // While there are still available stack frames:
        'outer: while let Some(mut stack_frame) = self.stack.pop() {
            // If we're at the end of the input:
            if stack_frame.grapheme_index == self.grapheme_bounds.len() {
                // If the input itself wasn't empty:
                if !self.input.is_empty() {
                    // Backtrack.
                    self.input_prefix_indexes.pop();
                // If the input itself was empty:
                } else {
                    // We're done, so we deallocate the stack (the only thing allocated
                    // when the input is empty) in case the programmer keeps this
                    // iterator around.
                    self.stack = Vec::new();

                    // Produce an empty Vec for an empty input.
                    return if n == 0 { Some(Vec::new()) } else { None };
                }
            // If we're not at the end of the input:
            } else {
                // Get the current input grapheme's end and the current NFD grapheme's
                // start and end.
                let input_nfd_grapheme_start_index = stack_frame
                    .grapheme_index
                    .checked_sub(1)
                    .map(|i| unsafe { self.grapheme_bounds.get_unchecked(i) }.input_nfd_index)
                    .unwrap_or(0);
                let GraphemeBounds {
                    input_index: input_grapheme_end_index,
                    input_nfd_index: input_nfd_grapheme_end_index,
                } = *unsafe {
                    self.grapheme_bounds
                        .get_unchecked(stack_frame.grapheme_index)
                };

                // For each byte in the current NFD grapheme:
                for byte in self.input_nfd
                    [input_nfd_grapheme_start_index..input_nfd_grapheme_end_index]
                    .bytes()
                {
                    // If there are words in the Dictionary that also have this byte of
                    // the NFD grapheme:
                    if let Some(transition_index) = stack_frame.current_node.find_input(byte) {
                        // Follow the path that includes this byte of the NFD grapheme
                        // as well.
                        stack_frame.current_node = self
                            .dictionary
                            .node(stack_frame.current_node.transition(transition_index).addr);
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is more backtracking left to be
                    // done:
                    } else if !self.stack.is_empty() {
                        // Backtrack.
                        self.input_prefix_indexes.pop();
                        continue 'outer;
                    // If there are no words in the Dictionary that also have this byte
                    // of the NFD grapheme and there is no backtracking left to be done:
                    } else {
                        // We're done, so we deallocate everything allocated in case
                        // the programmer keeps this iterator around.
                        self.input_nfd = String::new();
                        self.grapheme_bounds = Vec::new();
                        self.stack = Vec::new();
                        self.input_prefix_indexes = Vec::new();
                        return None;
                    }
                }

                // Adjust the end of this part of the prefix to the end of the current
                // NFD grapheme.
                *self.input_prefix_indexes.last_mut().unwrap() = input_grapheme_end_index;

                // If, after the current NFD grapheme is followed in the Dictionary, no
                // words end here:
                if !stack_frame.current_node.is_final() {
                    // Continue with the next NFD grapheme.
                    stack_frame.grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    self.stack.push(stack_frame);
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here and there are NFD graphemes left:
                } else if input_nfd_grapheme_end_index < self.input_nfd.len() {
                    // Recurse.
                    self.input_prefix_indexes.push(input_grapheme_end_index);

                    let grapheme_index = stack_frame.grapheme_index.wrapping_add(1);
                    stack_frame.grapheme_index = grapheme_index;
                    self.stack.push(stack_frame);
                    self.stack.push(StackFrame {
                        grapheme_index,
                        current_node: self.dictionary.root(),
                    });
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, there are no NFD graphemes left, and we've reached the nth
                // element:
                } else if n == 0 {
                    let input_prefix_indexes_iter = self.input_prefix_indexes.iter().copied();
                    let result = core::iter::once(0)
                        .chain(input_prefix_indexes_iter.clone())
                        .zip(input_prefix_indexes_iter)
                        .map(|(start, end)| &self.input[start..end])
                        .collect::<Vec<_>>();

                    // If there is some backtracking left to be done:
                    if !self.stack.is_empty() {
                        // Backtrack.
                        self.input_prefix_indexes.pop();
                    // If there's no remaining backtracking left to be done:
                    } else {
                        // We're done, so we deallocate everything allocated in case
                        // the programmer keeps this iterator around.
                        self.input_nfd = String::new();
                        self.grapheme_bounds = Vec::new();
                        self.stack = Vec::new();
                        self.input_prefix_indexes = Vec::new();
                    }

                    return Some(result);
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, there are no NFD graphemes left, we haven't reached the
                // nth element, and there is some backtracking left to be done:
                } else if !self.stack.is_empty() {
                    n = n.wrapping_sub(1);

                    // Backtrack.
                    self.input_prefix_indexes.pop();
                // If, after the current grapheme is followed in the Dictionary, a word
                // ends here, there are no NFD graphemes left, we haven't reached the
                // nth element, and there's no backtracking left to be done:
                } else {
                    // We're done, so we deallocate everything allocated in case
                    // the programmer keeps this iterator around.
                    self.input_nfd = String::new();
                    self.grapheme_bounds = Vec::new();
                    self.stack = Vec::new();
                    self.input_prefix_indexes = Vec::new();

                    return None;
                }
            }
        }

        // The work has all been done, so there are no more concatenations to produce.
        None
    }

    /* Nightly
    fn is_sorted(self) -> bool {
        true
    }
    */

    fn size_hint(&self) -> (usize, Option<usize>) {
        // If we're done producing all concatenations, there are zero more elements:
        if self.stack.is_empty() {
            (0, Some(0))
        // Otherwise, if the input is empty, there is exactly one concatenation.
        // Otherwise, the borders between graphemes can either be a word separation
        // point or not, meaning there are 2ⁿ⁻¹ possible elements, where n is the number
        // of graphemes:
        } else {
            self.grapheme_bounds
                .len()
                .checked_sub(1)
                .map(|grapheme_border_count| {
                    // Needed instead of usize::BITS because MSRV is 1.36 and usize::BITS didn't exist then
                    const USIZE_BITS: usize = 0_usize.count_zeros() as usize;

                    if grapheme_border_count < USIZE_BITS {
                        (0, Some(1 << grapheme_border_count))
                    } else {
                        (0, None)
                    }
                })
                .unwrap_or((1, Some(1)))
        }
    }
}

impl<'d, 's, D> core::iter::FusedIterator for Concatenations<'d, 's, D> where D: AsRef<[u8]> {}

#[derive(Clone, Debug)]
struct GraphemeBounds {
    input_index: usize,
    input_nfd_index: usize,
}

#[derive(Clone, Debug)]
struct StackFrame<'d> {
    grapheme_index: usize,
    current_node: Node<'d>,
}
