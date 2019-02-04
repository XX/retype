use std::sync::Arc;
use retype::retype;

#[retype(FooBar)]
type Bar = i32;

struct Foo {
    field: Bar,
}

#[retype(FooBarGen)]
type BarGen<T> = std::rc::Rc<T>;

struct FooGen<T> {
    field: BarGen<T>,
}

#[retype(FooDefaultBaz)]
type Baz = i32;

struct FooDefault {
    field: Baz,
}

#[test]
fn usage() {
    let a = Foo {
        field: 1.0,
    };

    let b = Foo {
        field: 2.0,
    };

    let c = FooDefault {
        field: 3,
    };

    let res = a.field + b.field;
    assert_eq!(3.0, res);

    let res: i32 = c.field;
    assert_eq!(3, res);

    let mut a = FooGen {
        field: BarGen::new(42),
    };
    a.field = Arc::new(42);

    let res = *a.field.as_ref();
    assert_eq!(42, res);
}