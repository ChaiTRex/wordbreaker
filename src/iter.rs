//TODO: remove
#![allow(dead_code)]

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitvec::vec::BitVec;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::Dictionary;

/// The <code>[Iterator](core::iter::Iterator)</code> that
/// <code>[Dictionary](crate::Dictionary)::[word_segmentations](crate::Dictionary::word_segmentations)</code>
/// produces.
#[derive(Clone, Debug)]
pub struct WordSegmentations<'s> {
    // TODO: Perhaps wrap everything except the two paths in a single Arc to make
    // cloning much cheaper.
    input: &'s str,
    word_boundary_str_indexes: Option<Arc<[usize]>>,
    solutions_tree: Option<Arc<BitVec>>,
    word_start_solution_subcounts: Option<Arc<[Option<core::num::NonZeroUsize>]>>,
    current_forward_path: Vec<(usize, core::ops::Range<usize>)>,
    current_reverse_path: Vec<(usize, core::ops::Range<usize>)>,
}

impl<'s> WordSegmentations<'s> {
    /*pub(crate) fn advance_back_by(&mut self, n: usize) -> Result<(), usize> {
        todo!()
    }*/

    pub(crate) fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        // Handle no advancement
        if n == 0 {
            return Ok(());
        }

        // Handle special situations
        if self.solutions_tree.is_none() {
            // Handle if iterator has no more items
            if self.current_forward_path.is_empty() {
                return Err(0);
            // Handle if iterator contains exactly one item: an empty Vec
            } else {
                self.current_forward_path = Vec::new();
                self.current_reverse_path = Vec::new();

                return if n == 1 { Ok(()) } else { Err(1) };
            }
        }

        // Find where the forward path and reverse path diverge.
        let mut starting_path_index = 0;
        loop {
            match (
                self.current_forward_path.get(starting_path_index),
                self.current_reverse_path.get(starting_path_index),
            ) {
                (Some(f), Some(r)) => {
                    // If the paths match, continue to the next node in both paths to
                    // find where they diverge:
                    if f.0 == r.0 {
                        starting_path_index = starting_path_index.wrapping_add(1);
                    // If the paths don't match, we've found where they diverge and can quit
                    // looking:
                    } else {
                        break;
                    }
                }
                // Handle if the full forward path is the same as the full reverse
                // path:
                (None, None) => {
                    self.word_boundary_str_indexes = None;
                    self.solutions_tree = None;
                    self.word_start_solution_subcounts = None;
                    self.current_forward_path = Vec::new();
                    self.current_reverse_path = Vec::new();

                    if n == 1 {
                        return Ok(());
                    } else {
                        return Err(1);
                    }
                }
                // If the forward and reverse paths have been the same and only one of
                // them suddenly ends, either the one that continues on continues past
                // the end of the `input` or the one that stops doesn't reach the end
                // of the `input`, neither of which should be possible:
                _ => unreachable!(),
            }
        }

        // Go forward until going forward some more will go past the nth item, then go
        // toward the end of string
        let mut advance_remaining = n;
        'faux_loop: loop {
            // Handle forward-specific subcounts
            for _ in starting_path_index.wrapping_add(1)..self.current_forward_path.len() {
                let (ref mut starting_node, ref mut unreached_range) =
                    self.current_forward_path.last_mut().unwrap();

                for node_offset in self
                    .solutions_tree
                    .as_ref()
                    .unwrap()
                    .get/*_unchecked*/(unreached_range.clone())
                    .unwrap()
                    .iter_ones()
                    .map(|node_offset_minus_one| node_offset_minus_one.wrapping_add(1))
                {
                    let next_node = starting_node.wrapping_add(node_offset);

                    let next_node_solution_count = self
                        .word_start_solution_subcounts
                        .as_ref()
                        .unwrap()
                        .get/*_unchecked*/(next_node)
                        .unwrap();

                    if next_node_solution_count.is_none()
                        || next_node_solution_count.unwrap().get() >= advance_remaining
                    {
                        unreached_range.start = unreached_range
                            .start
                            .wrapping_add(next_node)
                            .wrapping_sub(*starting_node);
                        *starting_node = next_node;
                        break 'faux_loop;
                    } else {
                        advance_remaining =
                            advance_remaining.wrapping_sub(next_node_solution_count.unwrap().get());
                    }
                }

                self.current_forward_path.pop();
            }

            // Handle middle subcounts
            let (current_forward_index, remaining_forward_range) =
                self.current_forward_path.get(starting_path_index).unwrap();
            let (_, remaining_reverse_range) =
                self.current_reverse_path.get(starting_path_index).unwrap();

            let middle_range = remaining_forward_range.start..remaining_reverse_range.end;
            for node_offset in self
                .solutions_tree
                .as_ref()
                .unwrap()
                .get/*_unchecked*/(middle_range)
                .unwrap()
                .iter_ones()
                .map(|node_offset_minus_one| node_offset_minus_one.wrapping_add(1))
            {
                let next_node = current_forward_index.wrapping_add(node_offset);

                let next_node_solution_count = self
                    .word_start_solution_subcounts
                    .as_ref()
                    .unwrap()
                    .get/*_unchecked*/(next_node)
                    .unwrap();

                if next_node_solution_count.is_none()
                    || next_node_solution_count.unwrap().get() >= advance_remaining
                {
                    let last_forward_segment = self.current_forward_path.last_mut().unwrap();

                    last_forward_segment.0 = next_node;
                    last_forward_segment.1.start =
                        last_forward_segment.1.start.wrapping_add(node_offset);

                    break 'faux_loop;
                } else {
                    advance_remaining =
                        advance_remaining.wrapping_sub(next_node_solution_count.unwrap().get());
                }
            }

            // Handle reverse-specific subcounts
            for (ending_node, unreached_range) in unsafe {
                self.current_reverse_path.get/*_unchecked*/(starting_path_index.wrapping_add(1)..).unwrap()
            }
            .iter()
            .cloned()
            {
                let starting_node_plus_one = ending_node
                    .wrapping_add(unreached_range.start)
                    .wrapping_sub(unreached_range.end);

                for next_node in unsafe {
                    self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(unreached_range).unwrap()
                }
                .iter_ones()
                .map(|offset_minus_one| {
                    starting_node_plus_one.wrapping_add(offset_minus_one)
                }) {
                    let next_node_solution_count = self
                        .word_start_solution_subcounts
                        .as_ref()
                        .unwrap()
                        .get/*_unchecked*/(next_node)
                        .unwrap();
                    if next_node_solution_count.is_none() ||
                        next_node_solution_count.unwrap().get() >= advance_remaining
                    {
                        let starting_node = next_node;
                        let starting_range = unsafe { get_row_bounds(starting_node, self.word_boundary_str_indexes.as_ref().unwrap().len()) };
                        self.current_forward_path.push((starting_node, starting_range));

                        break 'faux_loop;
                    } else {
                        advance_remaining =
                            advance_remaining.wrapping_sub(next_node_solution_count.unwrap().get());
                    }
                }
            }

            // We've reached the end of the iterator
            self.word_boundary_str_indexes = None;
            self.solutions_tree = None;
            self.word_start_solution_subcounts = None;
            self.current_forward_path = Vec::new();
            self.current_reverse_path = Vec::new();

            return Err(n.wrapping_sub(advance_remaining));
        }

        // Create rest of forward path
        todo!();

        Ok(())
    }

    pub(crate) fn new<D>(dictionary: &Dictionary<D>, input: &'s str) -> Self
    where
        D: AsRef<[u8]>,
    {
        let dictionary = &dictionary.fst;
        let dictionary_root = dictionary.root();

        if input.is_empty() {
            Self {
                input,
                word_boundary_str_indexes: None,
                solutions_tree: None,
                word_start_solution_subcounts: None,
                current_forward_path: vec![(0, 0..0)],
                current_reverse_path: vec![(0, 0..0)],
            }
        } else {
            let input_nfd = input.nfd().collect::<String>();
            let input_nfd_grapheme_indexes = input_nfd
                .grapheme_indices(true)
                .map(|(position, _)| position)
                .chain(core::iter::once(input_nfd.len()))
                .collect::<Vec<_>>();
            let old_node_count = input_nfd_grapheme_indexes.len();
            let old_last_node_index = old_node_count.wrapping_sub(1);
            let old_edge_count = old_node_count
                .wrapping_sub(!old_node_count & 1)
                .checked_mul(old_node_count >> 1)
                .expect("Too many input graphemes.");
            /*            println!("input_nfd: {input_nfd}");
                        println!("input_nfd_grapheme_indexes: {input_nfd_grapheme_indexes:?}");
                        println!("old_node_count: {old_node_count}");
                        println!("old_last_node_index: {old_last_node_index}");
                        println!("old_edge_count: {old_edge_count}");
            */
            let mut remaining_word_starts = <BitVec>::repeat(false, old_node_count);
            unsafe {
                remaining_word_starts.set/*_unchecked*/(0, true);
            }
            let mut old_solutions_tree = <BitVec>::repeat(false, old_edge_count);
            /*            println!("remaining_word_starts: {remaining_word_starts}");
                        println!("old_solutions_tree: {old_solutions_tree}");
            */
            while let Some(starting_node) = remaining_word_starts.first_one() {
                let mut cursor_node = dictionary_root;

                let ending_nodes = unsafe {
                    old_solutions_tree.get_mut/*_unchecked*/(get_row_bounds/*_unchecked*/(
                        starting_node,
                        old_last_node_index,
                    )).unwrap()
                };
                /*                println!("starting_node: {starting_node}");
                println!("get_row_bounds/*_unchecked*/
(starting_node, old_last_node_index): {:?}", unsafe { get_row_bounds/*_unchecked*/(starting_node, old_last_node_index) });
                println!("ending_nodes: {ending_nodes}");
*/                
                let node_grapheme_indexes_iter = unsafe {
                    input_nfd_grapheme_indexes.get/*_unchecked*/(starting_node..).unwrap()
                }
                .iter()
                .copied();

                'words_search: for (ending_node_offset_minus_one, input_nfd_grapheme) in
                    node_grapheme_indexes_iter
                        .clone()
                        .zip(node_grapheme_indexes_iter.skip(1))
                        .map(|(start, end)| unsafe {
                            input_nfd.get/*_unchecked*/(start..end).unwrap()
                        })
                        .enumerate()
                {
                    /*                    println!("ending_node_offset_minus_one: {ending_node_offset_minus_one}");
                    println!("input_nfd_grapheme: {input_nfd_grapheme}");*/
                    for byte in input_nfd_grapheme.bytes() {
                        if let Some(transition_index) = cursor_node.find_input(byte) {
                            //println!("byte: {byte} (accepted)");
                            cursor_node =
                                dictionary.node(cursor_node.transition_addr(transition_index));
                        } else {
                            //println!("byte: {byte} (rejected)");
                            break 'words_search;
                        }
                    }

                    if cursor_node.is_final() {
                        //println!("input_nfd_grapheme ends a word");
                        unsafe {
                            ending_nodes.set/*_unchecked*/(ending_node_offset_minus_one, true);
                        }
                        let ending_node = starting_node
                            .wrapping_add(ending_node_offset_minus_one)
                            .wrapping_add(1);
                        unsafe {
                            remaining_word_starts.set/*_unchecked*/(ending_node, true);
                        }
                        //println!("ending_nodes: {ending_nodes}");
                        //println!("ending_node: {ending_node}");
                    }
                }

                unsafe {
                    remaining_word_starts.set/*_unchecked*/(starting_node, false);
                }
                //println!("remaining_word_starts: {remaining_word_starts}");
            }
            //println!("old_solutions_tree: {old_solutions_tree}");

            let mut deleted_starting_nodes = <BitVec>::repeat(false, old_node_count);
            //println!("deleted_starting_nodes: {deleted_starting_nodes}");

            // Remove dead ends. With rows going from high to low:
            //    1. First, remove all rows until one ends with input_nfd's end.
            //    2. From there, remove all rows that have no word ends at all.
            // It should be noted that when a row is removed:
            //    1. It should be cleared of word ends if that's not already done.
            //    2. It should be removed from the word ends for every row less than it.
            let mut found_a_starting_node_that_reaches_end_of_string = false;

            for (starting_node, ending_nodes_range) in
                unsafe { row_bounds_rev_iter(old_node_count) }
            {
                //println!("starting_node: {starting_node}");
                //println!("ending_nodes_range: {ending_nodes_range:?}");
                let ending_nodes = unsafe {
                    old_solutions_tree.get_mut/*_unchecked*/(ending_nodes_range).unwrap()
                };
                //println!("ending_nodes: {ending_nodes}");

                //println!("found_a_starting_node_that_reaches_end_of_string: {found_a_starting_node_that_reaches_end_of_string}");
                if found_a_starting_node_that_reaches_end_of_string {
                    //println!("deleted_starting_nodes.get_mut/*_unchecked*/(starting_node.wrapping_add(1)..).unwrap(): {}", deleted_starting_nodes.get_mut/*_unchecked*/(starting_node.wrapping_add(1)..).unwrap());
                    for node_to_delete_offset_minus_one in unsafe {
                        deleted_starting_nodes.get_mut/*_unchecked*/(starting_node.wrapping_add(1)..).unwrap()
                    }
                    .iter_ones()
                    {
                        //println!("starting_node.wrapping_add(1): {}", starting_node.wrapping_add(1));
                        //println!("node_to_delete_offset_minus_one: {node_to_delete_offset_minus_one}");
                        unsafe {
                            ending_nodes.set/*_unchecked*/(node_to_delete_offset_minus_one, false);
                        }
                    }
                    if ending_nodes.is_empty() {
                        unsafe {
                            deleted_starting_nodes.set/*_unchecked*/(starting_node, true);
                        }
                    }
                    //println!("deleted_starting_nodes: {deleted_starting_nodes}");
                } else {
                    //println!("ending_nodes: {ending_nodes}");
                    //println!("*ending_nodes.last().unwrap(): {}", *ending_nodes.last().unwrap());
                    if *ending_nodes.last().unwrap() {
                        found_a_starting_node_that_reaches_end_of_string = true;
                        unsafe {
                            ending_nodes.split_at_mut/*_unchecked*/(ending_nodes.len().wrapping_sub(1))
                        }
                        .0
                        .fill(false);
                    } else {
                        ending_nodes.fill(false);
                        unsafe {
                            deleted_starting_nodes.set/*_unchecked*/(starting_node, true);
                        }
                    }
                }
                //println!("deleted_starting_nodes: {deleted_starting_nodes}");
                //println!("ending_nodes: {ending_nodes}");
                //println!("found_a_starting_node_that_reaches_end_of_string: {found_a_starting_node_that_reaches_end_of_string}");
            }

            let node_count = deleted_starting_nodes.count_zeros();
            //println!("node_count: {node_count}");
            if node_count <= 1 {
                return Self {
                    input,
                    word_boundary_str_indexes: None,
                    solutions_tree: None,
                    word_start_solution_subcounts: None,
                    current_forward_path: Vec::new(),
                    current_reverse_path: Vec::new(),
                };
            }

            let last_node_index = node_count.wrapping_sub(1);
            let edge_count = node_count
                .wrapping_sub(!node_count & 1)
                .wrapping_mul(node_count >> 1);

            let word_boundary_str_indexes;
            let mut solutions_tree;

            if node_count == old_node_count {
                word_boundary_str_indexes = input
                    .grapheme_indices(true)
                    .map(|(position, _)| position)
                    .chain(core::iter::once(input.len()))
                    .collect::<Vec<_>>();

                solutions_tree = old_solutions_tree;
            } else {
                word_boundary_str_indexes = input
                    .grapheme_indices(true)
                    .map(|(position, _)| position)
                    .enumerate()
                    .filter(|&(i, _)| !*unsafe {
                        deleted_starting_nodes.get/*_unchecked*/(i).unwrap()
                    })
                    .map(|(_, position)| position)
                    .chain(core::iter::once(input.len()))
                    .collect::<Vec<_>>();

                let mut old_to_new_indexes = vec![usize::MAX; old_node_count];
                for (new_index, old_index) in deleted_starting_nodes.iter_zeros().enumerate() {
                    *unsafe {
                        old_to_new_indexes.get_mut/*_unchecked*/(old_index).unwrap()
                    } = new_index;
                }

                solutions_tree = <BitVec>::repeat(false, edge_count);

                //println!("deleted_starting_nodes: {deleted_starting_nodes:?}");
                for (old_starting_index, (new_starting_index, ending_nodes_range)) in
                    deleted_starting_nodes
                        .iter_zeros()
                        .rev()
                        .skip(1)
                        .zip(unsafe { row_bounds_rev_iter(node_count) })
                {
                    //println!("old_starting_index: {old_starting_index}");
                    //println!("new_starting_index: {new_starting_index}");
                    //println!("ending_nodes_range: {ending_nodes_range:?}");
                    let old_ending_nodes = unsafe {
                        old_solutions_tree
                            .get/*_unchecked*/(get_row_bounds/*_unchecked*/(old_starting_index, old_last_node_index)).unwrap()
                    };
                    //println!("old_ending_nodes: {old_ending_nodes}");

                    let new_ending_nodes = unsafe {
                        solutions_tree.get_mut/*_unchecked*/(ending_nodes_range).unwrap()
                    };
                    //println!("new_ending_nodes: {new_ending_nodes}");

                    for old_ending_node in old_ending_nodes.iter_ones().map(|offset_minus_one| {
                        old_starting_index
                            .wrapping_add(offset_minus_one)
                            .wrapping_add(1)
                    }) {
                        //println!("old_ending_node: {old_ending_node}");
                        let new_ending_node_offset_minus_one = unsafe {
                            old_to_new_indexes.get/*_unchecked*/(old_ending_node).unwrap()
                        }
                        .wrapping_sub(new_starting_index)
                        .wrapping_sub(1);
                        //println!("new_ending_node: {new_ending_node}");

                        unsafe {
                            new_ending_nodes.set/*_unchecked*/(new_ending_node_offset_minus_one, true)
                        };
                    }
                }
            }

            let mut word_start_solution_subcounts = vec![None; node_count];
            *word_start_solution_subcounts.last_mut().unwrap() =
                Some(unsafe { core::num::NonZeroUsize::new_unchecked(1) });

            for (starting_node, ending_nodes_range) in unsafe { row_bounds_rev_iter(node_count) } {
                //println!("starting_node: {starting_node}");
                //println!("ending_nodes_range: {ending_nodes_range:?}");
                let ending_nodes = unsafe {
                    solutions_tree.get/*_unchecked*/(ending_nodes_range).unwrap()
                };

                let mut solution_count_overflow = false;
                let mut solution_count = 0_usize;

                for solution_subcount in ending_nodes.iter_ones().map(|offset_minus_one| {
                    word_start_solution_subcounts.get/*_unchecked*/(
                            starting_node.wrapping_add(offset_minus_one).wrapping_add(1)
                        ).unwrap()
                }) {
                    let solution_subcount = match solution_subcount {
                        Some(solution_subcount) => solution_subcount.get(),
                        None => {
                            solution_count_overflow = true;
                            break;
                        }
                    };
                    solution_count = match solution_count.checked_add(solution_subcount) {
                        Some(solution_count) => solution_count,
                        None => {
                            solution_count_overflow = true;
                            break;
                        }
                    };
                }

                let solution_count = if solution_count_overflow {
                    None
                } else {
                    Some(unsafe { core::num::NonZeroUsize::new_unchecked(solution_count) })
                };

                *unsafe {
                    word_start_solution_subcounts.get_mut/*_unchecked*/(starting_node).unwrap()
                } = solution_count;
            }

            let mut current_forward_path = Vec::new();
            let mut node = 0;
            while node != last_node_index {
                let mut row_bounds = unsafe {
                    get_row_bounds/*_unchecked*/(node, last_node_index)
                };
                let row = unsafe {
                    solutions_tree.get/*_unchecked*/(row_bounds.clone()).unwrap()
                };
                let next_offset = row.first_one().unwrap().wrapping_add(1);

                node = node.wrapping_add(next_offset);
                row_bounds.start = row_bounds.start.wrapping_add(next_offset);
                current_forward_path.push((node, row_bounds));
            }

            let mut current_reverse_path = Vec::new();
            let mut node = 0;
            while node != last_node_index {
                let mut row_bounds = unsafe {
                    get_row_bounds/*_unchecked*/(node, last_node_index)
                };
                let row = unsafe {
                    solutions_tree.get/*_unchecked*/(row_bounds.clone()).unwrap()
                };
                let next_offset_minus_one = row.last_one().unwrap();

                node = node.wrapping_add(next_offset_minus_one).wrapping_add(1);
                row_bounds.end = row_bounds.start.wrapping_add(next_offset_minus_one);
                current_reverse_path.push((node, row_bounds));
            }

            Self {
                input,
                word_boundary_str_indexes: Some(Arc::from(word_boundary_str_indexes)),
                solutions_tree: Some(Arc::new(solutions_tree)),
                word_start_solution_subcounts: Some(Arc::from(word_start_solution_subcounts)),
                current_forward_path,
                current_reverse_path,
            }
        }
    }
}

impl<'s> DoubleEndedIterator for WordSegmentations<'s> {
    /* TODO: nightly
    fn advance_back_by(&mut self, n: usize) -> Result<(), usize> {
        // Replace this with WordSegmentations::advance_back_by
    } */

    fn next_back(&mut self) -> Option<Self::Item> {
        // println!("self: {self:?}");
        if self.solutions_tree.is_none() {
            // println!("solutions_tree is empty");
            if self.current_reverse_path.is_empty() {
                // println!("current_forward_path is empty");
                None
            } else {
                // println!("current_forward_path is not empty");
                self.current_forward_path = Vec::new();
                self.current_reverse_path = Vec::new();

                Some(Vec::new())
            }
        } else {
            let iter = self
                .current_reverse_path
                .iter()
                .cloned()
                .map(|(input_index, _)| input_index);
            // println!("iter.clone().collect::<Vec<_>>(): {:?}", iter.clone().collect::<Vec<_>>());

            let result = core::iter::once(0)
                .chain(iter.clone())
                .zip(iter.clone())
                .map(|(start, end)| {
                    // println!("start: {start}");
                    // println!("end: {end}");
                    // println!("self.word_boundary_str_indexes: {:?}", self.word_boundary_str_indexes);
                    let start = *unsafe {
                        self.word_boundary_str_indexes.as_ref().unwrap().get/*_unchecked*/(start).unwrap()
                    };
                    let end = *unsafe {
                        self.word_boundary_str_indexes.as_ref().unwrap().get/*_unchecked*/(end).unwrap()
                    };
                    unsafe {
                        self.input.get/*_unchecked*/(start..end).unwrap()
                    }
                })
                .collect::<Vec<_>>();

            if iter.eq(self
                .current_forward_path
                .iter()
                .cloned()
                .map(|(input_index, _)| input_index))
            {
                self.word_boundary_str_indexes = None;
                self.solutions_tree = None;
                self.word_start_solution_subcounts = None;
                self.current_forward_path = Vec::new();
                self.current_reverse_path = Vec::new();
            } else {
                loop {
                    let last_on_path = self.current_reverse_path.last_mut().unwrap();
                    let (mut current_node, mut next_nodes_range) = last_on_path.clone();

                    let next_nodes = unsafe {
                        self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(next_nodes_range.clone()).unwrap()
                    };
                    match next_nodes.last_one().map(|forward_offset| {
                        next_nodes_range
                            .end
                            .wrapping_sub(next_nodes_range.start)
                            .wrapping_sub(forward_offset)
                    }) {
                        Some(offset) => {
                            current_node = current_node.wrapping_sub(offset);
                            next_nodes_range.end = next_nodes_range.end.wrapping_sub(offset);
                            *last_on_path = (current_node, next_nodes_range);

                            let last_node_index = self
                                .word_boundary_str_indexes
                                .as_ref()
                                .unwrap()
                                .len()
                                .wrapping_sub(1);

                            while current_node != last_node_index {
                                let mut row_bounds = unsafe {
                                    get_row_bounds/*_unchecked*/(current_node, last_node_index)
                                };
                                let row = unsafe {
                                    self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(row_bounds.clone()).unwrap()
                                };
                                let next_offset_minus_one = row.last_one().unwrap();

                                current_node = current_node
                                    .wrapping_add(next_offset_minus_one)
                                    .wrapping_add(1);
                                row_bounds.end =
                                    row_bounds.start.wrapping_add(next_offset_minus_one);
                                self.current_reverse_path.push((current_node, row_bounds));
                            }

                            break;
                        }
                        None => {
                            self.current_reverse_path.pop();
                        }
                    }
                }
            }

            Some(result)
        }
    }

    /*fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        match self.advance_back_by(n) {
            Ok(_) => self.next_back(),
            Err(_) => None,
        }
    }*/
}

impl<'s> core::iter::FusedIterator for WordSegmentations<'s> {}

impl<'s> Iterator for WordSegmentations<'s> {
    type Item = Vec<&'s str>;

    /* TODO: nightly
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        // Replace this with WordSegmentations::advance_by
    } */

    fn count(self) -> usize {
        #[cfg(debug_assertions)]
        {
            match self.size_hint().1 {
                Some(size) => size,
                None => panic!(
                    "The count of elements in this WordSegmentations iterator overflows a `usize`"
                ),
            }
        }
        #[cfg(not(debug_assertions))]
        {
            self.size_hint().0
        }
    }

    /* TODO: nightly
    fn is_sorted(self) -> bool {
        true
    } */

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn max(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        //println!("self: {self:?}");
        if self.solutions_tree.is_none() {
            //println!("solutions_tree is empty");
            if self.current_forward_path.is_empty() {
                //println!("current_forward_path is empty");
                None
            } else {
                //println!("current_forward_path is not empty");
                self.current_forward_path = Vec::new();
                self.current_reverse_path = Vec::new();

                Some(Vec::new())
            }
        } else {
            let iter = self
                .current_forward_path
                .iter()
                .cloned()
                .map(|(input_index, _)| input_index);
            //println!("iter.clone().collect::<Vec<_>>(): {:?}", iter.clone().collect::<Vec<_>>());

            let result = core::iter::once(0)
                .chain(iter.clone())
                .zip(iter.clone())
                .map(|(start, end)| {
                    //println!("start: {start}");
                    //println!("end: {end}");
                    //println!("self.word_boundary_str_indexes: {:?}", self.word_boundary_str_indexes);
                    let start = *unsafe {
                        self.word_boundary_str_indexes.as_ref().unwrap().get/*_unchecked*/(start).unwrap()
                    };
                    let end = *unsafe {
                        self.word_boundary_str_indexes.as_ref().unwrap().get/*_unchecked*/(end).unwrap()
                    };
                    unsafe {
                        self.input.get/*_unchecked*/(start..end).unwrap()
                    }
                })
                .collect::<Vec<_>>();

            if iter.eq(self
                .current_reverse_path
                .iter()
                .cloned()
                .map(|(input_index, _)| input_index))
            {
                self.word_boundary_str_indexes = None;
                self.solutions_tree = None;
                self.word_start_solution_subcounts = None;
                self.current_forward_path = Vec::new();
                self.current_reverse_path = Vec::new();
            } else {
                loop {
                    let last_on_path = self.current_forward_path.last_mut().unwrap();
                    let mut current_node = last_on_path.0;
                    let mut next_nodes_range = last_on_path.1.clone();

                    let next_nodes = unsafe {
                        self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(next_nodes_range.clone()).unwrap()
                    };
                    match next_nodes
                        .first_one()
                        .map(|offset_minus_one| offset_minus_one.wrapping_add(1))
                    {
                        Some(offset) => {
                            current_node = current_node.wrapping_add(offset);
                            next_nodes_range.start = next_nodes_range.start.wrapping_add(offset);

                            *last_on_path = (current_node, next_nodes_range);

                            let last_node_index = self
                                .word_boundary_str_indexes
                                .as_ref()
                                .unwrap()
                                .len()
                                .wrapping_sub(1);
                            while current_node != last_node_index {
                                next_nodes_range = unsafe {
                                    get_row_bounds/*_unchecked*/(current_node, last_node_index)
                                };

                                let offset = unsafe {
                                    self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(next_nodes_range.clone()).unwrap()
                                }
                                .first_one()
                                .unwrap()
                                .wrapping_add(1);
                                next_nodes_range.start =
                                    next_nodes_range.start.wrapping_add(offset);
                                current_node = current_node.wrapping_add(offset);

                                self.current_forward_path
                                    .push((current_node, next_nodes_range))
                            }

                            break;
                        }
                        None => {
                            self.current_forward_path.pop();
                        }
                    }
                }
            }

            Some(result)
        }
    }

    /*fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.advance_by(n) {
            Ok(_) => self.next(),
            Err(_) => None,
        }
    }*/

    fn size_hint(&self) -> (usize, Option<usize>) {
        // If there are no nonempty solutions
        if self.solutions_tree.is_none() {
            // If there are no solutions
            if self.current_forward_path.is_empty() {
                (0, Some(0))
            // If there is one empty-Vec solution
            } else {
                (1, Some(1))
            }
        // If there is at least one nonempty solution
        } else {
            let mut total_solution_count = 0_usize;

            let mut starting_path_index = 0;
            let mut solution_subcounts_iter = loop {
                match (
                    self.current_forward_path.get(starting_path_index),
                    self.current_reverse_path.get(starting_path_index),
                ) {
                    (
                        Some((forward_path_node, forward_range)),
                        Some((reverse_path_node, reverse_range)),
                    ) => {
                        // Keep following the paths if they're the same so far
                        if forward_path_node == reverse_path_node {
                            starting_path_index = starting_path_index.wrapping_add(1);
                        // If the paths differ, start counting solutions
                        } else {
                            break unsafe {
                                // Get the index range in the solution tree between the
                                // forward and reverse paths
                                self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(forward_range.start..reverse_range.end).unwrap()
                            }
                            // Go through the solution tree indexes that represent the
                            // next node in the solution
                            .iter_ones()
                            // Convert each solution tree index to the solution
                            // subcount for the node it represents
                            .map(move |node_offset_minus_one| {
                                self.word_start_solution_subcounts.as_ref().unwrap().get/*_unchecked*/(
                                    forward_path_node.wrapping_add(node_offset_minus_one).wrapping_add(1)
                                ).unwrap()
                            });
                        }
                    }
                    // If the forward and reverse paths are the same and reach the end
                    // of the string, they share a solution that's the last solution
                    (None, None) => return (1, Some(1)),
                    _ => unreachable!("A path that reaches the end of the string is not the prefix of a different path that reaches the end of the string."),
                }
            };

            // Handle forward-specific subcounts
            for (starting_node, unreached_range) in unsafe {
                self.current_forward_path.get/*_unchecked*/(starting_path_index.wrapping_add(1)..).unwrap()
            }
            .iter()
            .cloned()
            {
                let starting_node_plus_one = starting_node.wrapping_add(1);
                for unreached_node in unsafe {
                    self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(unreached_range).unwrap()
                }
                .iter_ones()
                .map(|offset_minus_one| {
                    starting_node_plus_one.wrapping_add(offset_minus_one)
                }) {
                    let solution_subcount = match unsafe {
                        self.word_start_solution_subcounts.as_ref().unwrap().get/*_unchecked*/(unreached_node).unwrap()
                    } {
                        Some(solution_subcount) => solution_subcount.get(),
                        None => return (usize::MAX, None),
                    };
                    total_solution_count =
                        match total_solution_count.checked_add(solution_subcount) {
                            Some(solution_subcount) => solution_subcount,
                            None => return (usize::MAX, None),
                        };
                }
            }

            // Add one for end of string
            total_solution_count = match total_solution_count.checked_add(1) {
                Some(solution_subcount) => solution_subcount,
                None => return (usize::MAX, None),
            };

            // Handle subcounts strictly between the two nodes the forward and reverse
            // paths first differ on
            let solution_subcount = match solution_subcounts_iter.next() {
                Some(Some(solution_subcount)) => solution_subcount.get(),
                Some(None) => return (usize::MAX, None),
                None => 0,
            };
            total_solution_count = match total_solution_count.checked_add(solution_subcount) {
                Some(solution_count) => solution_count,
                None => return (usize::MAX, None),
            };

            for solution_subcount in solution_subcounts_iter {
                let solution_subcount = match solution_subcount {
                    Some(solution_subcount) => solution_subcount.get(),
                    None => return (usize::MAX, None),
                };
                total_solution_count = match total_solution_count.checked_add(solution_subcount) {
                    Some(solution_subcount) => solution_subcount,
                    None => return (usize::MAX, None),
                };
            }

            // Handle reverse-specific subcounts
            for (ending_node, unreached_range) in unsafe {
                self.current_reverse_path.get/*_unchecked*/(starting_path_index.wrapping_add(1)..).unwrap()
            }
            .iter()
            .cloned()
            {
                let starting_node_plus_one = ending_node
                    .wrapping_add(unreached_range.start)
                    .wrapping_sub(unreached_range.end);

                for unreached_node in unsafe {
                    self.solutions_tree.as_ref().unwrap().get/*_unchecked*/(unreached_range).unwrap()
                }
                .iter_ones()
                .map(|offset_minus_one| {
                    starting_node_plus_one.wrapping_add(offset_minus_one)
                }) {
                    let solution_subcount = match unsafe {
                        self.word_start_solution_subcounts.as_ref().unwrap().get/*_unchecked*/(unreached_node).unwrap()
                    } {
                        Some(solution_subcount) => solution_subcount.get(),
                        None => return (usize::MAX, None),
                    };
                    total_solution_count =
                        match total_solution_count.checked_add(solution_subcount) {
                            Some(solution_subcount) => solution_subcount,
                            None => return (usize::MAX, None),
                        };
                }
            }

            // Add one for end of string
            let total_solution_count = match total_solution_count.checked_add(1) {
                Some(solution_subcount) => solution_subcount,
                None => return (usize::MAX, None),
            };

            return (total_solution_count, Some(total_solution_count));
        }
    }
}

unsafe fn get_row_bounds(starting_node: usize, last_node_index: usize) -> core::ops::Range<usize> {
    let nodes_after_this = last_node_index.wrapping_sub(starting_node);
    let start = nodes_after_this.wrapping_mul(nodes_after_this.wrapping_sub(1)) >> 1;
    let end = start.wrapping_add(nodes_after_this);

    start..end
}

unsafe fn row_bounds_rev_iter(
    node_count: usize,
) -> impl Iterator<Item = (usize, core::ops::Range<usize>)> {
    (0..node_count.wrapping_sub(1)).rev().zip({
        (1..node_count).scan(0, |boundary: &mut usize, offset| {
            let start = *boundary;
            *boundary = boundary.wrapping_add(offset);

            Some(start..*boundary)
        })
    })
}

unsafe fn row_bounds_iter(
    node_count: usize,
    edge_count: usize,
) -> impl Iterator<Item = (usize, core::ops::Range<usize>)> {
    (0..node_count.wrapping_sub(1)).zip({
        (1..node_count)
            .rev()
            .scan(edge_count, |boundary: &mut usize, offset| {
                let end = *boundary;
                *boundary = boundary.wrapping_sub(offset);

                Some(*boundary..end)
            })
    })
}

/* TODO: nightly
unsafe impl<'s> core::iter::TrustedLen for WordSegmentations<'s> {}
*/
