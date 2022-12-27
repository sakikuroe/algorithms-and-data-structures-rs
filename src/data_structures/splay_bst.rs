#[derive(Clone, PartialEq, Eq)]
pub struct SplayNode<T>
where
    T: Ord + Clone,
{
    key: T,
    left: *mut Self,
    right: *mut Self,
    parent: *mut Self,
    size: usize,
}

impl<T> SplayNode<T>
where
    T: Ord + Clone,
{
    pub fn new(key: T) -> Self {
        SplayNode {
            key,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            size: 1,
        }
    }

    pub fn update(&mut self) {
        unsafe {
            self.size = 1;
            if !self.left.is_null() {
                self.size += (*(self.left)).size;
            }

            if !self.right.is_null() {
                self.size += (*(self.right)).size;
            }
        }
    }

    pub fn rotate(&mut self) {
        unsafe {
            let mut p = self.parent;
            let mut pp = (*p).parent;
            let mut c;

            if (*p).left == self {
                c = self.right;
                self.right = p;
                (*p).left = c;
            } else {
                c = self.left;
                self.left = p;
                (*p).right = c;
            }

            if !pp.is_null() {
                if (*pp).left == p {
                    (*pp).left = self;
                }
                if (*pp).right == p {
                    (*pp).right = self;
                }
            }

            self.parent = pp;
            (*p).parent = self;
            if !c.is_null() {
                (*c).parent = p;
            }

            (*p).update();
            self.update();
        }
    }

    pub fn state(&self) -> isize {
        unsafe {
            if !self.parent.is_null() && std::ptr::eq((*self.parent).left, self) {
                1
            } else if !self.parent.is_null() && std::ptr::eq((*self.parent).right, self) {
                -1
            } else {
                0
            }
        }
    }

    pub fn splay(&mut self) {
        unsafe {
            while !self.parent.is_null() {
                if (*self.parent).state() == 0 {
                    // zig
                    self.rotate();
                } else if (*self).state() == (*self.parent).state() {
                    // zig-zag
                    (*self.parent).rotate();
                    (*self).rotate();
                } else {
                    // zig-zig
                    (*self).rotate();
                    (*self).rotate();
                }
            }
        }
    }

    pub fn get_nth(&mut self, mut idx: usize) -> *mut SplayNode<T> {
        let mut root = self as *mut SplayNode<T>;
        unsafe {
            loop {
                let lsize = if !(*root).left.is_null() {
                    (*(*root).left).size
                } else {
                    0
                };
                match idx.cmp(&lsize) {
                    std::cmp::Ordering::Less => {
                        root = (*root).left;
                    }
                    std::cmp::Ordering::Equal => {
                        (*root).splay();
                        return root;
                    }
                    std::cmp::Ordering::Greater => {
                        root = (*root).right;
                        idx = idx - lsize - 1;
                    }
                }
            }
        }
    }

    pub fn lower_bound(&mut self, x: T) -> usize {
        let root = self as *mut SplayNode<T>;
        unsafe {
            if x <= (*root).key {
                if (*root).left.is_null() {
                    return 0;
                } else {
                    (*(*root).left).lower_bound(x)
                }
            } else {
                (if (*root).left.is_null() {
                    0
                } else {
                    (*(*root).left).size
                }) + if (*root).right.is_null() {
                    0
                } else {
                    (*(*root).right).lower_bound(x)
                } + 1
            }
        }
    }

    pub fn split(&mut self, idx: usize) -> (*mut SplayNode<T>, *mut SplayNode<T>) {
        let mut root = self as *mut SplayNode<T>;
        unsafe {
            if idx == 0 {
                return (std::ptr::null_mut(), root);
            }
            if idx == (*root).size {
                return (root, std::ptr::null_mut());
            }
            root = (*root).get_nth(idx);
            let mut l_root = (*root).left;
            let mut r_root = root;
            (*r_root).left = std::ptr::null_mut();
            (*l_root).parent = std::ptr::null_mut();
            (*r_root).update();
            (l_root, r_root)
        }
    }

    pub fn insert(&mut self, node: *mut SplayNode<T>) -> *mut SplayNode<T> {
        let root = self as *mut SplayNode<T>;
        let idx = unsafe { (*root).lower_bound((*node).key.clone()) };
        let (l_root, r_root) = unsafe { (*root).split(idx) };
        merge(merge(l_root, node), r_root)
    }

    pub fn remove(&mut self, idx: usize) -> (*mut SplayNode<T>, *mut SplayNode<T>) {
        let mut root = self as *mut SplayNode<T>;
        unsafe {
            root = (*root).get_nth(idx);
            let mut l_root = (*root).left;
            let mut r_root = (*root).right;
            if !l_root.is_null() {
                (*l_root).parent = std::ptr::null_mut()
            };
            if !r_root.is_null() {
                (*r_root).parent = std::ptr::null_mut()
            };

            (*root).left = std::ptr::null_mut();
            (*root).right = std::ptr::null_mut();
            (*root).update();
            (merge(l_root, r_root), root)
        }
    }
}

fn merge<T>(mut l_root: *mut SplayNode<T>, mut r_root: *mut SplayNode<T>) -> *mut SplayNode<T>
where
    T: Ord + Clone,
{
    unsafe {
        if l_root.is_null() {
            r_root
        } else if r_root.is_null() {
            l_root
        } else {
            l_root = (*l_root).get_nth((*l_root).size - 1);
            (*l_root).right = r_root;
            (*r_root).parent = l_root;
            (*l_root).update();
            l_root
        }
    }
}

pub struct SplayBST<T>
where
    T: Ord + Clone,
{
    pub root: *mut SplayNode<T>,
}

impl<T> SplayBST<T>
where
    T: Ord + Clone,
{
    pub fn new() -> Self {
        SplayBST {
            root: std::ptr::null_mut(),
        }
    }

    pub fn insert(&mut self, x: T) {
        let add = Box::into_raw(Box::new(SplayNode::new(x)));
        if (self.root as *mut SplayNode<T>).is_null() {
            self.root = add;
        } else {
            unsafe {
                self.root = (*(*self).root).insert(add);
            }
        }
    }

    pub fn get_nth(&mut self, n: usize) -> Option<T> {
        unsafe {
            if (*self).root.is_null() {
                return None;
            }
            if (*(*self).root).size <= n {
                return None;
            }
            self.root = (*self.root).get_nth(n);
            if self.root.is_null() {
                None
            } else {
                Some((*self.root).key.clone())
            }
        }
    }

    pub fn remove(&mut self, x: T) {
        unsafe {
            let idx = (*(*self).root).lower_bound(x);
            self.root = (*(*self).root).remove(idx).0;
        }
    }

    pub fn contains(&mut self, key: &T) -> bool {
        let mut root = self.root as *mut SplayNode<T>;
        unsafe {
            loop {
                if root.is_null() {
                    return false;
                }
                if (*root).key == *key {
                    return true;
                }
                if *key <= (*root).key {
                    if (*root).left.is_null() {
                        return false;
                    } else {
                        root = (*root).left;
                    }
                } else {
                    if (*root).right.is_null() {
                        return false;
                    } else {
                        root = (*root).right;
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        if (self.root as *mut SplayNode<T>).is_null() {
            0
        } else {
            unsafe { (*self.root).size }
        }
    }
}
