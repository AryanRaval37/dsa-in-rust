use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

pub struct Bst<T> {
    head: Link<T>
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {  
    elem: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T: Ord> Bst<T> {
    pub fn new() -> Self {
        Self {head: None}
    }

    pub fn insert(&mut self, elem: T) {
        // at each node, if elem > value then go right, if less then go left, if equal done.
        // when you can't go down anymore then add it there and end.
        let mut link = &mut self.head;

        // traverse down the tree to the end
        while let Some(node) = link {
            match elem.cmp(&node.elem) {
                Ordering::Less => link = &mut node.left,
                Ordering::Greater => link = &mut node.right,
                Ordering::Equal => return,
            }
        }
        // now that we are at correct place, just add the next node after link.
        *link = Some(Box::new(Node {
            elem: elem,
            right: None,
            left: None,
        }));
    }

    pub fn search(&self, elem: T) -> bool {
        let mut link = &self.head;
        while let Some(node) = link {
            match elem.cmp(&node.elem) {
                Ordering::Less => link = &node.left,
                Ordering::Greater => link = &node.right,
                Ordering::Equal => return true,
            }
        }
        return false;
    }
}

// ! AI generated print function for debug
impl<T: fmt::Display> fmt::Display for Bst<T> {
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
                        positions[&(*node as *const Node<T>)] * half_cell_width
                            + half_cell_width;

                    if let Some(left) = node.left.as_deref() {
                        let child_position =
                            positions[&(left as *const Node<T>)] * half_cell_width
                                + half_cell_width;

                        largest_distance =
                            largest_distance.max(parent_position.abs_diff(child_position));
                    }

                    if let Some(right) = node.right.as_deref() {
                        let child_position =
                            positions[&(right as *const Node<T>)] * half_cell_width
                                + half_cell_width;

                        largest_distance =
                            largest_distance.max(parent_position.abs_diff(child_position));
                    }
                }

                // One slash row is normally enough.
                // Add another row only when the gap is unusually wide.
                let max_distance_per_branch_row = cell_width ;

                let branch_rows = if largest_distance <= max_distance_per_branch_row {
                    1
                } else {
                    (largest_distance + max_distance_per_branch_row - 1)
                        / max_distance_per_branch_row
                };

                for branch_step in 1..=branch_rows.max(1) {
                    let mut branch_line = vec![' '; line_width];

                    for node in level {
                        let parent_position =
                            positions[&(*node as *const Node<T>)] * half_cell_width
                                + half_cell_width;

                        if let Some(left) = node.left.as_deref() {
                            let child_position =
                                positions[&(left as *const Node<T>)] * half_cell_width
                                    + half_cell_width;

                            let position = parent_position as isize
                                + (child_position as isize - parent_position as isize)
                                    * branch_step as isize
                                    / (branch_rows + 1) as isize;

                            branch_line[position as usize] = '/';
                        }

                        if let Some(right) = node.right.as_deref() {
                            let child_position =
                                positions[&(right as *const Node<T>)] * half_cell_width
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
    use super::Bst;

}