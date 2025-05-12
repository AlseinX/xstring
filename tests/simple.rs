use std::{fmt::Debug, ops::Index};

use xstring::{Str, XString};

const SHORT: &str = "hello world!";
const LONG: &str = "Some very very long long string value";

#[test]
fn inline() {
    play_with(XString::new(SHORT), SHORT, 1..SHORT.len() - 1)
}

#[test]
fn alloc() {
    play_with(XString::new(LONG), LONG, 1..SHORT.len() - 1)
}

#[test]
fn r#static() {
    play_with(XString::from_static(LONG), LONG, 1..SHORT.len() - 1)
}

fn play_with<T: Str + ?Sized + PartialEq + Debug, R: Clone>(x: XString<T>, s: &T, range: R)
where
    T: Index<R, Output = T>,
{
    assert_eq!(x, s);
    assert_eq!(x.slice(range.clone()), s[range]);
    let y = x.clone();
    assert_eq!(x, y);
    drop(x);
    assert_eq!(y, s);
}
