use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

pub struct Avl<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    left: Link<T>,
    right: Link<T>,
    height: u32,
}

impl<T> Node<T> {
    // added functions to calculate the balance of the node
    // right heavy means positive
    fn balance(&self) -> i32 {
        match (&self.left, &self.right) {
            (Some(x), Some(y)) => return y.height as i32 - x.height as i32,
            (Some(x), None) => return -(x.height as i32),
            (None, Some(y)) => return y.height as i32,
            (None, None) => return 0,
        }
    }

    fn update_height(&mut self) {
        let mut left_height = 0;
        let mut right_height = 0;
        if let Some(x) = self.left.as_ref() {
            left_height = x.height
        }
        if let Some(y) = self.right.as_ref() {
            right_height = y.height;
        }
        self.height = 1 + left_height.max(right_height);
    }
}

pub struct Iter<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T> Iter<'a, T> {
    // storing the whole left subtree references to nodes
    pub fn new(bst: &'a Avl<T>) -> Self {
        let mut iter = Iter { stack: Vec::new() };
        iter.push_left_path(bst.head.as_deref());
        iter
    }

    // function that pushes the whole left path from the current node into the stack of the iter
    fn push_left_path(&mut self, mut node: Option<&'a Node<T>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.push_left_path(node.right.as_deref());
        Some(&node.elem)
    }
}

impl<T> Avl<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }
}

impl<T: Ord> Avl<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    /*
     *    x                 y
     *   / \               / \
     *  A   y      ->     x   C
     *     / \           / \
     *    B   C         A   B
     */
    fn rotate_left(link: &mut Link<T>) {
        let Some(mut x) = link.take() else {
            return;
        };
        let Some(mut y) = x.right.take() else {
            // nothing to rotate link just points to a node with no children
            // put back the node in the link and return
            *link = Some(x);
            return;
        };
        // only x and y's children have changed to update their heights;
        x.right = y.left.take();
        x.update_height();
        y.left = Some(x);
        y.update_height();

        *link = Some(y);
    }

    /*
     *      x                 y
     *     / \               / \
     *    y   C      ->     A   x
     *   / \                   / \
     *  A   B                 B   C
     */
    fn rotate_right(link: &mut Link<T>) {
        let Some(mut x) = link.take() else {
            return;
        };
        let Some(mut y) = x.left.take() else {
            // nothing to rotate link just points to a node with no children
            // put back the node in the link and return
            *link = Some(x);
            return;
        };
        // only x and y's children have changed to update their heights;
        x.left = y.right.take();
        x.update_height();
        y.right = Some(x);
        y.update_height();

        *link = Some(y);
    }

    fn rebalance(link: &mut Link<T>) {
        let Some(node) = link.as_mut() else {
            return;
        };
        node.update_height();

        // Balance is right height minus left height.
        if node.balance() < -1 {
            let left = node.left.as_ref().unwrap();
            if left.balance() > 0 {
                // Left-right case: straighten the left child first.
                Self::rotate_left(&mut node.left);
            }
            Self::rotate_right(link);
        } else if node.balance() > 1 {
            let right = node.right.as_ref().unwrap();
            if right.balance() < 0 {
                // Right-left case: straighten the right child first.
                Self::rotate_right(&mut node.right);
            }
            Self::rotate_left(link);
        }
    }

    pub fn insert(&mut self, elem: T) {
        Self::insert_at(&mut self.head, elem);
    }

    fn insert_at(link: &mut Link<T>, elem: T) {
        let Some(node) = link.as_mut() else {
            *link = Some(Box::new(Node {
                elem,
                left: None,
                right: None,
                height: 1,
            }));
            return;
        };

        match elem.cmp(&node.elem) {
            Ordering::Less => Self::insert_at(&mut node.left, elem),
            Ordering::Greater => Self::insert_at(&mut node.right, elem),
            Ordering::Equal => return, // Keep just one copy of each value.
        }

        // Recursion repairs the children before we repair their parent.
        Self::rebalance(link);
    }

    // returns the value elem if it is found else returns None
    pub fn search(&self, elem: T) -> Option<T> {
        let mut link = &self.head;
        while let Some(node) = link {
            match elem.cmp(&node.elem) {
                Ordering::Less => link = &node.left,
                Ordering::Greater => link = &node.right,
                Ordering::Equal => return Some(elem),
            }
        }
        return None;
    }

    // Returns the stored value if it exists in the tree.
    pub fn delete(&mut self, elem: T) -> Option<T> {
        Self::delete_at(&mut self.head, &elem)
    }

    // vide coded delete_at
    // whatever. it was painful to implement avl trees
    fn delete_at(link: &mut Link<T>, elem: &T) -> Option<T> {
        let node = link.as_mut()?;

        let deleted = match elem.cmp(&node.elem) {
            Ordering::Less => Self::delete_at(&mut node.left, elem),
            Ordering::Greater => Self::delete_at(&mut node.right, elem),
            Ordering::Equal => {
                let mut removed = link.take().unwrap();

                if removed.left.is_none() {
                    *link = removed.right.take();
                } else if removed.right.is_none() {
                    *link = removed.left.take();
                } else {
                    // Move the smallest node in the right subtree into this spot.
                    let mut successor = Self::remove_min(&mut removed.right);
                    successor.left = removed.left.take();
                    successor.right = removed.right.take();
                    *link = Some(successor);
                }

                Some(removed.elem)
            }
        };

        Self::rebalance(link);
        deleted
    }

    // The caller must provide a nonempty subtree.
    fn remove_min(link: &mut Link<T>) -> Box<Node<T>> {
        let node = link
            .as_mut()
            .expect("remove_min requires a nonempty subtree");

        if node.left.is_some() {
            let smallest = Self::remove_min(&mut node.left);
            Self::rebalance(link);
            smallest
        } else {
            let mut smallest = link.take().unwrap();
            *link = smallest.right.take();
            smallest
        }
    }
}

// ! AI generated print function for debug
impl<T: fmt::Display> fmt::Display for Avl<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(root) = self.head.as_deref() else {
            return write!(f, "(empty)");
        };

        // Gather nodes by tree depth.
        let mut levels: Vec<Vec<&Node<T>>> = Vec::new();
        let mut current_level = vec![root];
        let mut widest_value = 1;

        while !current_level.is_empty() {
            let mut next_level = Vec::new();

            for node in &current_level {
                widest_value = widest_value.max(node.elem.to_string().chars().count());

                if let Some(left) = node.left.as_deref() {
                    next_level.push(left);
                }

                if let Some(right) = node.right.as_deref() {
                    next_level.push(right);
                }
            }

            levels.push(current_level);
            current_level = next_level;
        }

        // Post-order traversal: children appear before their parent.
        let mut postorder = Vec::new();
        let mut traversal_stack = vec![root];

        while let Some(node) = traversal_stack.pop() {
            postorder.push(node);

            if let Some(left) = node.left.as_deref() {
                traversal_stack.push(left);
            }

            if let Some(right) = node.right.as_deref() {
                traversal_stack.push(right);
            }
        }

        // Calculate how much horizontal space each subtree needs.
        let mut widths: HashMap<*const Node<T>, usize> = HashMap::new();

        for &node in postorder.iter().rev() {
            let left_width = node
                .left
                .as_deref()
                .map(|left| widths[&(left as *const Node<T>)])
                .unwrap_or(0);

            let right_width = node
                .right
                .as_deref()
                .map(|right| widths[&(right as *const Node<T>)])
                .unwrap_or(0);

            let width = match (node.left.as_deref(), node.right.as_deref()) {
                (None, None) => 1,
                (Some(_), None) => left_width + 1,
                (None, Some(_)) => right_width + 1,
                (Some(_), Some(_)) => left_width + right_width,
            };

            widths.insert(node as *const Node<T>, width);
        }

        // Assign each subtree a starting horizontal position.
        let mut starts: HashMap<*const Node<T>, usize> = HashMap::new();
        let mut placement_stack = vec![(root, 0usize)];

        while let Some((node, start)) = placement_stack.pop() {
            starts.insert(node as *const Node<T>, start);

            match (node.left.as_deref(), node.right.as_deref()) {
                (Some(left), Some(right)) => {
                    let left_width = widths[&(left as *const Node<T>)];
                    placement_stack.push((right, start + left_width));
                    placement_stack.push((left, start));
                }
                (Some(left), None) => {
                    placement_stack.push((left, start));
                }
                (None, Some(right)) => {
                    placement_stack.push((right, start + 1));
                }
                (None, None) => {}
            }
        }

        // Calculate the center position of every node.
        let mut positions: HashMap<*const Node<T>, usize> = HashMap::new();

        for &node in postorder.iter().rev() {
            let position = match (node.left.as_deref(), node.right.as_deref()) {
                (None, None) => 2 * starts[&(node as *const Node<T>)] + 1,
                (Some(left), None) => positions[&(left as *const Node<T>)] + 1,
                (None, Some(right)) => positions[&(right as *const Node<T>)] - 1,
                (Some(left), Some(right)) => {
                    let left_position = positions[&(left as *const Node<T>)];
                    let right_position = positions[&(right as *const Node<T>)];
                    (left_position + right_position) / 2
                }
            };

            positions.insert(node as *const Node<T>, position);
        }

        let mut cell_width = widest_value + 4;

        if cell_width % 2 != 0 {
            cell_width += 1;
        }

        let half_cell_width = cell_width / 2;
        let tree_width = widths[&(root as *const Node<T>)];
        let line_width = (tree_width + 2) * cell_width;

        for (depth, level) in levels.iter().enumerate() {
            let mut node_line = vec![' '; line_width];

            for node in level {
                let center =
                    positions[&(*node as *const Node<T>)] * half_cell_width + half_cell_width;

                let value = node.elem.to_string();
                let start = center.saturating_sub(value.chars().count() / 2);

                for (offset, character) in value.chars().enumerate() {
                    if start + offset < node_line.len() {
                        node_line[start + offset] = character;
                    }
                }
            }

            if depth > 0 {
                writeln!(f)?;
            }

            write!(
                f,
                "{}",
                node_line.into_iter().collect::<String>().trim_end()
            )?;

            if depth + 1 < levels.len() {
                let mut largest_distance = 0;

                for node in level {
                    let parent_position =
                        positions[&(*node as *const Node<T>)] * half_cell_width + half_cell_width;

                    if let Some(left) = node.left.as_deref() {
                        let child_position = positions[&(left as *const Node<T>)] * half_cell_width
                            + half_cell_width;

                        largest_distance =
                            largest_distance.max(parent_position.abs_diff(child_position));
                    }

                    if let Some(right) = node.right.as_deref() {
                        let child_position = positions[&(right as *const Node<T>)]
                            * half_cell_width
                            + half_cell_width;

                        largest_distance =
                            largest_distance.max(parent_position.abs_diff(child_position));
                    }
                }

                // One slash row is normally enough.
                // Add another row only when the gap is unusually wide.
                let max_distance_per_branch_row = cell_width;

                let branch_rows = if largest_distance <= max_distance_per_branch_row {
                    1
                } else {
                    (largest_distance + max_distance_per_branch_row - 1)
                        / max_distance_per_branch_row
                };

                for branch_step in 1..=branch_rows.max(1) {
                    let mut branch_line = vec![' '; line_width];

                    for node in level {
                        let parent_position = positions[&(*node as *const Node<T>)]
                            * half_cell_width
                            + half_cell_width;

                        if let Some(left) = node.left.as_deref() {
                            let child_position = positions[&(left as *const Node<T>)]
                                * half_cell_width
                                + half_cell_width;

                            let position = parent_position as isize
                                + (child_position as isize - parent_position as isize)
                                    * branch_step as isize
                                    / (branch_rows + 1) as isize;

                            branch_line[position as usize] = '/';
                        }

                        if let Some(right) = node.right.as_deref() {
                            let child_position = positions[&(right as *const Node<T>)]
                                * half_cell_width
                                + half_cell_width;

                            let position = parent_position as isize
                                + (child_position as isize - parent_position as isize)
                                    * branch_step as isize
                                    / (branch_rows + 1) as isize;

                            branch_line[position as usize] = '\\';
                        }
                    }

                    writeln!(f)?;
                    write!(
                        f,
                        "{}",
                        branch_line.into_iter().collect::<String>().trim_end()
                    )?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Avl, Node};

    // vide coded insert, delete, rotate tests
    fn assert_avl(tree: &Avl<i32>) {
        fn check(node: Option<&Node<i32>>, lower: Option<i32>, upper: Option<i32>) -> u32 {
            let Some(node) = node else {
                return 0;
            };

            if let Some(lower) = lower {
                assert!(node.elem > lower);
            }
            if let Some(upper) = upper {
                assert!(node.elem < upper);
            }

            let left_height = check(node.left.as_deref(), lower, Some(node.elem));
            let right_height = check(node.right.as_deref(), Some(node.elem), upper);
            let height = 1 + left_height.max(right_height);

            assert_eq!(node.height, height, "wrong height at {}", node.elem);
            assert!(
                left_height.abs_diff(right_height) <= 1,
                "unbalanced node: {}",
                node.elem
            );
            height
        }

        check(tree.head.as_deref(), None, None);
    }

    #[test]
    fn insertion_rotations() {
        for values in [[3, 2, 1], [1, 2, 3], [3, 1, 2], [1, 3, 2]] {
            let mut tree = Avl::new();
            for value in values {
                tree.insert(value);
                assert_avl(&tree);
            }

            assert_eq!(tree.head.as_ref().unwrap().elem, 2);
            assert_eq!(tree.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn duplicate_insertion() {
        let mut tree = Avl::new();
        for value in [2, 1, 3, 2, 1, 3] {
            tree.insert(value);
            assert_avl(&tree);
        }
        assert_eq!(tree.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn deletion_rotations() {
        // Single rotations, double rotations, and a heavy child with balance zero.
        let cases: &[(&[i32], i32, i32)] = &[
            (&[3, 2, 4, 1], 4, 2),
            (&[2, 1, 3, 4], 1, 3),
            (&[5, 2, 6, 3], 6, 3),
            (&[2, 1, 5, 4], 1, 4),
            (&[4, 2, 5, 1, 3], 5, 2),
            (&[2, 1, 4, 3, 5], 1, 4),
        ];
        for &(values, deleted, root) in cases {
            let mut tree = Avl::new();
            for &value in values {
                tree.insert(value);
            }

            assert_eq!(tree.delete(deleted), Some(deleted));
            assert_avl(&tree);
            assert_eq!(tree.search(deleted), None);
            assert_eq!(tree.head.as_ref().unwrap().elem, root);
        }
    }

    #[test]
    fn sorted_insertions_and_deletions() {
        let mut tree = Avl::new();
        for value in 0..256 {
            tree.insert(value);
            assert_avl(&tree);
        }
        assert_eq!(tree.delete(-1), None);
        assert_avl(&tree);

        for value in 0..256 {
            assert_eq!(tree.delete(value), Some(value));
            assert_avl(&tree);
            assert_eq!(
                tree.iter().copied().collect::<Vec<_>>(),
                ((value + 1)..256).collect::<Vec<_>>()
            );
        }
        assert!(tree.head.is_none());
    }

    #[test]
    fn all_small_insertion_and_deletion_orders() {
        fn check_orders(values: &mut [i32], start: usize) {
            if start == values.len() {
                let mut tree = Avl::new();
                for &value in values.iter() {
                    tree.insert(value);
                    assert_avl(&tree);
                }
                assert_eq!(
                    tree.iter().copied().collect::<Vec<_>>(),
                    vec![0, 1, 2, 3, 4, 5]
                );

                for &value in values.iter() {
                    assert_eq!(tree.delete(value), Some(value));
                    assert_avl(&tree);
                    assert_eq!(tree.search(value), None);
                }
                assert!(tree.head.is_none());
                return;
            }

            for index in start..values.len() {
                values.swap(start, index);
                check_orders(values, start + 1);
                values.swap(start, index);
            }
        }

        check_orders(&mut [0, 1, 2, 3, 4, 5], 0);
    }

    #[test]
    // poor test, i know but whatever, im lazy, move on to next thing
    fn basics() {
        let mut bst: Avl<i32> = Avl::new();
        assert_eq!(bst.search(37), None);
        bst.insert(1);
        bst.insert(2);
        bst.insert(3);
        assert_eq!(bst.search(3), Some(3));
    }

    #[test]
    fn delete() {
        let mut bst: Avl<i32> = Avl::new();
        assert_eq!(bst.delete(37), None);
        bst.insert(1);
        bst.insert(2);
        bst.insert(3);
        assert_eq!(bst.delete(2), Some(2));
    }

    #[test]
    fn iter() {
        let mut bst = Avl::new();
        bst.insert(2);
        bst.insert(3);
        bst.insert(1);

        let mut iter = bst.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
    }
}
