pub fn remove_bracket(s: &Vec<char>) -> Vec<char> {
    let mut que = vec![];
    for c in s {
        if *c == ')' && que.len() >= 1 && que[que.len() - 1] == '(' {
            que.pop();
            que.pop();
        } else {
            que.push(*c);
        }
    }

    que
}

// ARC108-B
pub fn remove_fox(s: &Vec<char>) -> Vec<char> {
    let mut que = vec![];
    for c in s {
        if *c == 'x' && que.len() >= 2 && que[que.len() - 2..] == ['f', 'o'] {
            que.pop();
            que.pop();
        } else {
            que.push(*c);
        }
    }

    que
}
