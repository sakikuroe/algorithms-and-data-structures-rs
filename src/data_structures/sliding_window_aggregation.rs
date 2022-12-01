use crate::algebraic_structures::semi_group::SemiGroup;

type Stack<T> = Vec<T>;

pub struct FoldableStack<SG>
where
    SG: SemiGroup,
{
    stack: Stack<SG::S>,
    sum: Stack<SG::S>,
}

impl<SG> FoldableStack<SG>
where
    SG: SemiGroup,
    SG::S: Clone,
{
    pub fn new() -> Self {
        FoldableStack {
            stack: Stack::new(),
            sum: Stack::new(),
        }
    }

    pub fn push(&mut self, x: SG::S) {
        self.stack.push(x.clone());
        if self.sum.is_empty() {
            self.sum.push(x);
        } else {
            self.sum.push(SG::op(&x, &self.sum.last().unwrap()));
        }
    }

    pub fn pop(&mut self) -> Option<SG::S> {
        self.sum.pop();
        self.stack.pop()
    }

    pub fn fold(&self) -> Option<SG::S> {
        self.sum.last().cloned()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

pub struct FoldableStackRev<SG>
where
    SG: SemiGroup,
{
    stack: Stack<SG::S>,
    sum: Stack<SG::S>,
}

impl<SG> FoldableStackRev<SG>
where
    SG: SemiGroup,
    SG::S: Clone,
{
    pub fn new() -> Self {
        FoldableStackRev {
            stack: Stack::new(),
            sum: Stack::new(),
        }
    }

    pub fn push(&mut self, x: SG::S) {
        self.stack.push(x.clone());
        if self.sum.is_empty() {
            self.sum.push(x);
        } else {
            self.sum.push(SG::op(&self.sum.last().unwrap(), &x));
        }
    }

    pub fn pop(&mut self) -> Option<SG::S> {
        self.sum.pop();
        self.stack.pop()
    }

    pub fn fold(&self) -> Option<SG::S> {
        self.sum.last().cloned()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

pub struct FoldableDeque<SG>
where
    SG: SemiGroup,
{
    stack_front: FoldableStack<SG>,
    stack_back: FoldableStackRev<SG>,
}

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 146 F - Sugoroku](https://atcoder.jp/contests/abc146/tasks/abc146_f) ([submittion](https://atcoder.jp/contests/abc146/submissions/36905646))

impl<SG> FoldableDeque<SG>
where
    SG: SemiGroup,
    SG::S: Clone,
{
    pub fn new() -> Self {
        FoldableDeque {
            stack_front: FoldableStack::new(),
            stack_back: FoldableStackRev::new(),
        }
    }

    pub fn push_front(&mut self, x: SG::S) {
        self.stack_front.push(x);
    }

    pub fn push_back(&mut self, x: SG::S) {
        self.stack_back.push(x);
    }

    pub fn pop_front(&mut self) -> Option<SG::S> {
        if self.stack_front.is_empty() {
            for i in (0..(self.stack_back.len() + 1) / 2).rev() {
                self.stack_front.push(self.stack_back.stack[i].clone());
            }
            let temp = self.stack_back.stack[(self.stack_back.len() + 1) / 2..].to_vec();
            self.stack_back = FoldableStackRev::new();
            for x in temp {
                self.stack_back.push(x);
            }
        }

        self.stack_front.pop()
    }

    pub fn pop_back(&mut self) -> Option<SG::S> {
        if self.stack_back.is_empty() {
            for i in (0..(self.stack_front.len() + 1) / 2).rev() {
                self.stack_back.push(self.stack_front.stack[i].clone());
            }
            let temp = self.stack_front.stack[(self.stack_front.len() + 1) / 2..].to_vec();
            self.stack_front = FoldableStack::new();
            for x in temp {
                self.stack_front.push(x);
            }
        }

        self.stack_back.pop()
    }

    pub fn fold(&self) -> Option<SG::S> {
        match (self.stack_front.fold(), self.stack_back.fold()) {
            (Some(sum1), Some(sum2)) => Some(SG::op(&sum1, &sum2)),
            (Some(sum1), None) => Some(sum1),
            (None, Some(sum2)) => Some(sum2),
            (None, None) => None,
        }
    }

    pub fn len(&self) -> usize {
        self.stack_front.len() + self.stack_back.len()
    }
}
