/// verified by
/// - AtCoder | パナソニックグループプログラミングコンテスト2022（AtCoder Beginner Contest 273） E - Notebook
///     - [problem link](https://atcoder.jp/contests/abc273/tasks/abc273_e)
///     - [submittion](https://atcoder.jp/contests/abc273/submissions/35754512)
use std::rc::Rc;

#[derive(Clone)]
pub struct PersistentStackNode {
    value: usize,
    next: Option<Rc<PersistentStackNode>>,
}

#[derive(Clone)]
pub struct PersistentStack {
    head: Option<Rc<PersistentStackNode>>,
}

impl PersistentStack {
    pub fn push(&self, x: usize) -> PersistentStack {
        PersistentStack {
            head: Some(Rc::new(PersistentStackNode {
                value: x,
                next: self.head.clone(),
            })),
        }
    }

    pub fn top(&self) -> Option<usize> {
        if self.head.is_none() {
            None
        } else {
            Some(self.head.as_ref().unwrap().value)
        }
    }

    pub fn pop(&self) -> PersistentStack {
        if self.head.is_none() {
            self.clone()
        } else {
            PersistentStack {
                head: self.head.as_ref().unwrap().next.clone(),
            }
        }
    }
}
