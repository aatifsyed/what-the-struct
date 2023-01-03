use itertools::Itertools as _;
use std::path::Path;

fn do_test(parent: &[&str], children: &[&[&str]]) {
    let krate =
        what_the_struct::rustdoc("nightly", Path::new("test-crates/minimal/Cargo.toml")).unwrap();
    let id = krate
        .paths
        .iter()
        .flat_map(|(id, summary)| match summary.path == parent {
            true => Some(id),
            false => None,
        })
        .exactly_one()
        .unwrap();

    let (_, actual) = what_the_struct::struct_parent_and_children(&krate, id);
    assert_eq!(children, actual);
}

#[test]
fn plain_struct() {
    do_test(
        &["minimal", "PlainStruct"],
        &[
            &["core", "primitive", "bool"],
            &["core", "primitive", "i8"],
            &["minimal", "EmptyPlainStruct"],
            &["minimal", "nested", "UnitStruct"],
            &["core", "primitive", "char"],
            &["core", "primitive", "u8"],
        ],
    )
}
