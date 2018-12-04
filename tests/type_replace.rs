extern crate type_replacer;

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
        field: 1.into(),
    };

    let b = Foo {
        field: 2.into(),
    };

    let res = a.field + b.field;
    let exp: Bar = (1 + 2).into();

    assert_eq!(exp, res);

    let a = FooGen {
        field: BarGen::new(42),
    };

    let res = *a.field.as_ref();

    assert_eq!(42, res);
}