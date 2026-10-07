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

    pub fn insert(&mut self, elem: T) {}

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

    // returns the deleted value (same as elem) if it exists in the tree else returns None
    pub fn delete(&mut self, elem: T) -> Option<T> {
        let mut link = &mut self.head;
        loop {
            /* Observe
             * match link {
             *   None => return None, // not found
             *   Some(node) => {
             *       match elem.cmp(&node.elem) {
             *           Ordering::Less => link = &mut node.left,
             *           Ordering::Greater => link = &mut node.right,
             *           Ordering::Equal => break,
             *       }
             *   }
             * }
             * The problem here is that when link is matched against Some(Node),
             * node now mutably borrows the link so link cannot be mutated anymore
             * thats fine for less and greater because you put another mutable borrow right back in the link
             * however for the Equal case, link doesn't get anything back (link is currently immutable)
             */
            match link.as_ref().map(|node| elem.cmp(&node.elem)) {
                None => return None, // reached end of list, element not found
                Some(Ordering::Equal) => break,
                Some(Ordering::Less) => link = &mut link.as_mut().unwrap().left, // we know unwrap is safe here because we already checked if the node is None
                Some(Ordering::Greater) => link = &mut link.as_mut().unwrap().right,
            }
        }
        // node is found the element is at the node owned by current link
        // taking ownership of the boxed node by link.take(), making link = None
        let boxed = link.take().unwrap(); // this will definitely be Some because we know that the node here is the elem itself to be deleted.
        let deleted = boxed.elem;
        match (boxed.left, boxed.right) {
            (None, None) => { // case 1: leaf node, just delete it
                // nothing needs to be done here link is already None
            }
            (Some(child), None) | (None, Some(child)) => {
                // case 2: one child only
                // just delete the node at link and reconnect child to node above to the node above.
                *link = Some(child); // making the link which was None to the child of the deleted node, giving back ownership here.
            }
            (Some(left), Some(right)) => {
                // case 3: both children present, replace the current position with successor
                let mut right_link = Some(right);
                let mut cur = &mut right_link;
                while cur.as_ref().unwrap().left.is_some() {
                    cur = &mut cur.as_mut().unwrap().left;
                } // cur will now point to to leftmost node of right subtree
                let mut succ = cur.take().unwrap(); // take the successor node out of the tree
                *cur = succ.right.take(); // replace the successor node with its right child (if any)
                succ.left = Some(left); // connect the left child of the deleted node to the successor
                succ.right = right_link; // connect the right child of the deleted node to the successor
                *link = Some(succ); // replace the deleted node with the successor
            }
        };
        return Some(deleted);
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
    use super::Avl;

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
