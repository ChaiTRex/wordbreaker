use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;
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
    prefix: Vec<Range<usize>>,
}

impl<'d, 's, D> Concatenations<'d, 's, D>
where
    D: AsRef<[u8]>,
{
    pub(crate) fn new(dictionary: &'d Dictionary<D>, input: &'s str) -> Self {
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

    /*
    /* Nightly
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        // TODO: When this is stabilized, move `.nth(n)` code here and base `.nth()` on
        // this method.
    }
    */

    fn count(mut self) -> usize {
        todo!()
    }

    fn last(mut self) -> Option<Self::Item> {
        todo!()
    }

    #[inline(always)]
    fn max(mut self) -> Option<Self::Item> {
        self.last()
    }

    #[inline(always)]
    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
    */

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
                                #[allow(clippy::mem_replace_with_default)]
                                core::mem::replace(&mut self.prefix, Vec::new())
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

    /*
    /* Nightly
    fn is_sorted(self) -> bool {
        true
    }
    */

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
    */
}

impl<'d, 's, D> core::iter::FusedIterator for Concatenations<'d, 's, D> where D: AsRef<[u8]> {}

#[derive(Clone)]
struct GraphemeBounds {
    input_index: usize,
    input_nfd_index: usize,
}

#[derive(Clone)]
struct StackFrame<'d> {
    grapheme_end_index: usize,
    current_node: Node<'d>,
}
