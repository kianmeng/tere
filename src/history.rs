use std::cell::RefCell;
use std::rc::{Rc, Weak};


// Tree struct based on https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html
pub struct HistoryTreeEntry {
    name: String, //TODO: use Path / PathComponent instead? or None? to represent root (and what else?) correctly
    parent: Weak<Self>, // option is not needed (I guess), we can just use a null weak to represent the root
    last_visited_child: Option<Weak<Self>>,
    children: RefCell<Vec<Rc<Self>>>,
}

struct HistoryTree {
    root: Rc<HistoryTreeEntry>,
    current_entry: RefCell<Rc<HistoryTreeEntry>>,
}

impl HistoryTree {

    pub fn current_entry(&self) -> Rc<HistoryTreeEntry> {
        self.current_entry.borrow().clone()
    }

    pub fn visit(&mut self, fname: &str) {
        let matching_child = self.current_entry.borrow().children.borrow().iter()
            .find(|child| child.name == fname).map(|c| c.clone());

        if let Some(child) = matching_child {

            let mut previous_entry = self.current_entry.replace(Rc::clone(&child));
            Rc::get_mut(&mut previous_entry).unwrap().last_visited_child = Some(Rc::downgrade(&child));

        } else {
            let child = HistoryTreeEntry {
                name: fname.to_string(),
                parent: Rc::downgrade(&self.current_entry.borrow()),
                children: RefCell::new(vec![]),
                last_visited_child: None,
            };
            let child = Rc::new(child);
            self.current_entry.borrow_mut().children.borrow_mut().push(Rc::clone(&child));
            self.current_entry = RefCell::new(child);
        }
    }

    pub fn go_up(&mut self) {
        let maybe_parent = self.current_entry.borrow().parent.upgrade();
        if let Some(parent) = maybe_parent {
            self.current_entry = RefCell::new(Rc::clone(&parent));
        } // if the parent is None, we're at the root, so no need to do anything
    }

}

#[cfg(test)]
mod tests_for_history_tree {
    use super::*;

    fn init_history_tree() -> HistoryTree {
        let root = Rc::new(HistoryTreeEntry {
            name: "/".to_string(),
            parent: Weak::new(),
            last_visited_child: None,
            children: RefCell::new(vec![]),
        });

        HistoryTree {
            root: Rc::clone(&root),
            current_entry: RefCell::new(root),
        }
    }

    #[test]
    fn test_history_tree_visit() {
        let mut tree = init_history_tree();

        tree.visit("foo");
        assert_eq!(tree.current_entry().name, "foo");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().name, "/");

        tree.visit("bar");
        assert_eq!(tree.current_entry().name, "bar");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().name, "foo");
        assert_eq!(tree.current_entry().parent.upgrade().unwrap().parent.upgrade().unwrap().name, "/");

    }

    #[test]
    fn test_history_tree_go_up_down() {
        let mut tree = init_history_tree();

        tree.visit("foo");
        tree.visit("bar");

        tree.go_up();
        assert_eq!(tree.current_entry().name, "foo");
        assert_eq!(tree.current_entry().children.borrow()[0].name, "bar");

        tree.go_up();
        assert_eq!(tree.current_entry().name, "/");
        assert_eq!(tree.current_entry().children.borrow()[0].name, "foo");

        tree.go_up();
        assert_eq!(tree.current_entry().name, "/");
        assert_eq!(tree.current_entry().children.borrow()[0].name, "foo");

    }

    #[test]
    fn test_tree_pointer_counts() {
        let mut tree = init_history_tree();
        tree.visit("foo");
        let foo = Rc::downgrade(&tree.current_entry());
        tree.visit("bar");
        let bar = Rc::downgrade(&tree.current_entry());

        assert_eq!(Rc::weak_count(&tree.root), 1); // the child (foo)

        assert_eq!(Weak::strong_count(&foo), 1); // the root
        assert_eq!(Weak::weak_count(&foo), 2); // the child and the variable 'foo' above

        assert_eq!(Weak::strong_count(&bar), 2); // the parent (foo) and the tree current entry
        assert_eq!(Weak::weak_count(&bar), 1); // the variable 'bar' above

        tree.go_up(); tree.go_up();
        assert_eq!(Weak::strong_count(&bar), 1); // the parent only now
        assert_eq!(Weak::weak_count(&bar), 1); // the variable 'bar' above

        tree.visit("baz");
        assert_eq!(Rc::weak_count(&tree.root), 2); // two children

    }

}
