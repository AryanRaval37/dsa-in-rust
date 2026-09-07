use std::mem;

pub struct List<T> {
    head: Link<T>
}
enum Link<T> {
    Empty,
    More(Box<Node<T>>),
}

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {head: Link::Empty}
    }

    // function to just add an element just after the head
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });
        self.head = Link::More(new_node);
    }

    // popping the current element at the head.
    pub fn pop(&mut self) -> Option<T> {
        let result;
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => {
                result = None;
            },
            Link::More(node) => {
                result = Some(node.elem);
                self.head = node.next;
            },
        };
        result
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // set cur_link to the head of the list and clear out the head
        // here clear out the head means that self.head is an empty link, when the next iter of loop runs, cur link gets updated
        // no one owns the link and it gets drop automatically.
        // the clever idea is just to one by one make all the links empty  so that no one owns the nodes and everything gets droppped with it goes out of scope
        // This is not the equivalent to just popping every element of list because pop has to read and return the value and if its a bit value then its too expensive do bother about the actual data in the node.
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut boxed_node) = cur_link {
            // update the curlink to the next link and empty out the curren link.
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check if pop behaves with empty list
        assert_eq!(list.pop(), None);

        // add some elements
        list.push(1);
        list.push(2);
        list.push(3);

        // check removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // push more stuff to see nothings gone wrong
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

}

