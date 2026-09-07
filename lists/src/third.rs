/* 
Persistent Lists:
You want to be able to manipulate the tail of the lists
list1 = A -> B -> C -> D
list2 = tail(list1) = B -> C -> D
list3 = push(list2, X) = X -> B -> C -> D

We want memory to look like this:
list1 -> A ---+
              |
              v
list2 ------> B -> C -> D
              ^
              |
list3 -> X ---+

Problem: Who owms B? previously B was owned by A but shared ownership?
Box wont work, box can have only one owner

Use RC 'reference counted' pointers
You can have multiple references own the memory and the memory will be freed only when all of the go out of scope
Still only one mutable one though
*/

/* Note on Thread Safety
Rc is not thread safe, if you replace Rc everywhere by Arc then this list will be thread safe
Basically every type implements Send and Sync traits. Its just for other api to see there is nothing actually going on inside it.
A type is Send if its safe to move to another thread.
A type is Sync if its safe to share between multiply threads.
- Cells, atomics, interior mutability, rust thread safety.
*/


use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

// replace box with Rc
type Link<T> = Option<Rc<Node<T>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>
}

// code for iterator remains same
// IterMut and intoiter not possible because we only have shared access to elements.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref()
        }
    }

    // adds element to the front of the list and gives the new list back
    // now two things are looking at the second element of the new list (which is the same as the first element of self)
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem: elem,
                // clone is a trait implemented by a lot of types
                // in Rc, clone means to create a copy of 
                next: self.head.clone(),
            }))
        }
    }

    // returns the list leaving the current element
    // completed recheck : list.prepend(37).tail() will give back he same list (though multiple copies will the there)
    pub fn tail(&self) -> List<T> {
        List {
            // when you have type U returned then use map to match 
            // but if you're getting an option back then map will wrap another option around it
            // use and_then instead
            head: self.head.as_ref().and_then(|node| {
                node.next.clone()
            }),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        // as long as there are more nodes
        while let Some(node) = head {
            // Rc::try_unwrap returns the node if there is exactly one strong reference to it.
            // else returns error.
            // we want to clear out the node only only if this list owns it and not any other.
            if let Ok(mut node) = Rc::try_unwrap(node) {
                // clearing it out, head will be replaced by the next thing in the loop so this reference will go out of scope
                head = node.next.take();
            } else {
                // if try_unwrap returns an error meaning there are other lists also owning the node
                // then its the other lists responsibility to clean up from here.
                break;
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);

    }
}
