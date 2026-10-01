pub struct List<T> {
    head: Link<T>
}
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}


// struct wrapper around List
// 'Into Iter' make it to an iterator (list no more usable)
pub struct IntoIter<T>(List<T>);

// basically every struct and function which has a reference has to have a lifetimes associated with it
// (sometimes for functions this is implicit)
// this is saying that the Option has a borrowed reference therefore you have to give it a lfietime
// because this is part of the Iter struct, Iter also has this lifetime associated with it.
// (thos &Node<T> can't die while the Iter is still alive or else Iter will an invalid reference)
pub struct Iter<'a, T> {
    // why hold node reference?? why not link
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        // here self.next.take() is required which was not required for the non mut Iter
        // that is because of the magic of copy
        // types like i32, bool, char etc are Copy, which is that there is no ownership
        // they are just values, they can be moved or copied or anything
        // similarly for & also, &Node which is a reference to Node is just copy-able every no problems there
        // multiply copies can be stored and passed around no problem (view only)
        // however there can only one mutable reference therefore it doesn't have any copy-able thing
        // Option<&mut Node>> here needs be borrowed and replaced with None
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // remember map is replacement for match statement, if node is none then none
        // otherwise do whatever is in the function and leave (here also returns &node.elem)
        // so if the next node from where the iterator is is not None then return 
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {head: None}
    }

    // Function to return an iterator for the list
    // no lifetime for list but this function has a lifetime which it requires
    // &self has to outlive Iter that is returned, name both with lifetime a.
    // this iter<'a> comes from the function caller, it checks the scope of the variable that this reference is going into
    // sees how long the variable is used and finds its lifetime and passes it to the function
    // the function then returns this lifetime into the variable
    // pub fn iter<'a>(&'a self) -> Iter<'a, T> {
    // this is a more idiomatic way of writing it
    pub fn iter(&self) -> Iter<'_, T> {
        // create an iter from the head and return it
        Iter {
            /* 
            The next node is the first thing that we want to return which should be this head's node
            self.head is Option<Box<Node<T>>> but we want next to have Option<&Node<T>>
            as_ref() does to an option: takes the internals of the option and returns an option of a reference to that thing in the option
            as_deref() then derefences it from the box and gives the pointer of the actual node
            This is equivalent to 
            match &self.head {
                Some(boxed_Node) => Some(&**boxed_node)
                None => None
            }
            Here &self.head becaues you can only take reference not ownership,
            unpack the option -> &Box<Node<T>> here the & comes because you took &self.head and it creates references to all the owned data also.
            dereference once to get data there dereference again to remove the Box to get to Node<T> and then give a referene to that
            */
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut()
        }
    }

    // here just self is used meaning you're taking ownership.
    // gives back not a list but an iterator
    // only thing iterator has is next which give out the elements of the list one by one
    // internally it uses the list pop function.
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    // function to just add an element just after the head
    // list.push(1); list.push(2); list.push(3) will make the list head -> 3 -> 2 -> 1 -> None
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    // popping the current element at the head.
    pub fn pop(&mut self) -> Option<T> {
        // |node| is an inline function (a closure) 
        // map is for replacing the match statement.
        // take access to link which belons to head, map it to next node if there is a next node and return an element.
        // remember link is an option
        // map is a method for the option
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    // to return reference to data inside the current node at head.
    // pop is same but pop will give you the element and also remove it from the list.
    // as ref is an option thing to convert &Option<T> to Option<&T> 
    // the map then looks at the option unpacks it, if its actually an option first of all (self.head could be None)
    // then it takes the node from Some(node) (inline function called) gets &T back and wraps that in an option and returns.
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
    self.head.as_mut().map(|node| {
        &mut node.elem
    })
}
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // set cur_link to the head of the list and clear out the head
        // here clear out the head means that self.head is an empty link, when the next iter of loop runs, cur link gets updated
        // no one owns the link and it gets drop automatically.
        // the clever idea is just to one by one make all the links empty  so that no one owns the nodes and everything gets droppped with it goes out of scope
        // This is not the equivalent to just popping every element of list because pop has to read and return the value and if its a bit value then its too expensive do bother about the actual data in the node.
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            // update the curlink to the next link and empty out the curren link.
            cur_link = boxed_node.next.take();
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();

        // Note: interesting fact here and has come up
        // comparison in rust is very nice, iter.next() is an option
        // rust while comparing will 'open up' options and references to compare actual values
        // magic of PartialEq trait
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
}


    #[test]
    fn peaking() {
        let mut list = List::new();
        // peeking test
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1); list.push(2); list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
    }

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

        // Into Iter test
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

}

