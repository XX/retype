extern crate type_replacer;

use std::sync::Arc;
use type_replacer::replace;

#[replace(FooBar)]
type Bar = i32;

struct Foo {
    field: Bar,
}

#[replace(FooBarGen)]
type BarGen<T> = std::rc::Rc<T>;

struct FooGen<T> {
    field: BarGen<T>,
}

#[test]
fn usage() {
    let a = Foo {
        field: 1.0,
    };

    let b = Foo {
        field: 2.0,
    };

    let res = a.field + b.field;
    assert_eq!(3.0, res);

    let mut a = FooGen {
        field: BarGen::new(42),
    };
    a.field = Arc::new(42);

    let res = *a.field.as_ref();
    assert_eq!(42, res);
}