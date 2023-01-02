use std::collections::HashMap;

/// # Panics
/// If `id` point does not refer to a struct, enum or union
pub fn struct_parent_and_children<'a>(
    krate: &'a rustdoc_types::Crate,
    id: &rustdoc_types::Id,
) -> (&'a Vec<String>, Vec<&'a Vec<String>>) {
    use rustdoc_types::{ItemEnum, ItemKind, Struct, StructKind, Variant};

    let summary = &krate.paths[id];
    let parent = match &summary.kind {
        ItemKind::Enum | ItemKind::Struct | ItemKind::Union => &summary.path,
        other => panic!("Expected `Enum | Struct | Union`, not {other:?}"),
    };

    let struct_fields: Vec<&rustdoc_types::Type> = match &krate.index[id].inner {
        ItemEnum::Enum(enum_) => enum_
            .variants
            .iter()
            .map(|id| match &krate.index[id].inner {
                ItemEnum::Variant(Variant::Plain(_)) => vec![], // no children from this variant
                ItemEnum::Variant(Variant::Tuple(fields)) => fields
                    .iter()
                    .flat_map(|maybe_id| maybe_id)
                    .map(unwrap_struct_field(krate))
                    .collect(),
                ItemEnum::Variant(Variant::Struct { fields, .. }) => {
                    fields.iter().map(unwrap_struct_field(krate)).collect()
                }
                other => panic!("expected `Variant`, not {other:?}"),
            })
            .flatten()
            .collect(),
        ItemEnum::Struct(Struct {
            kind: StructKind::Unit,
            ..
        }) => vec![], // no fields in this struct
        ItemEnum::Struct(Struct {
            kind: StructKind::Plain { fields, .. },
            ..
        }) => fields.iter().map(unwrap_struct_field(krate)).collect(),
        ItemEnum::Struct(Struct {
            kind: StructKind::Tuple(fields),
            ..
        }) => fields
            .iter()
            .flat_map(|maybe_id| maybe_id)
            .map(unwrap_struct_field(krate))
            .collect(),
        ItemEnum::Union(union) => union
            .fields
            .iter()
            .map(unwrap_struct_field(krate))
            .collect(),
        other => panic!("expected `Enum | Struct | Union`, not `{other:?}`"),
    };

    let struct_fields = struct_fields
        .into_iter()
        .flat_map(|type_| get_resolved_paths(krate, type_))
        .collect();

    (parent, struct_fields)
}

fn unwrap_struct_field<'a>(
    krate: &'a rustdoc_types::Crate,
) -> impl Fn(&rustdoc_types::Id) -> &'a rustdoc_types::Type {
    |id| match &krate.index[id].inner {
        rustdoc_types::ItemEnum::StructField(field) => field,
        other => panic!("expected `StructField`, not {other:?}"),
    }
}

// TODO(aatifsyed): will this handle recursive types?
fn get_resolved_paths<'a>(
    krate: &'a rustdoc_types::Crate,
    type_: &rustdoc_types::Type,
) -> Vec<&'a Vec<String>> {
    use rustdoc_types::Type;

    match type_ {
        Type::ResolvedPath(path) => vec![&krate.paths[&path.id].path],
        Type::DynTrait(_) => vec![],
        Type::Generic(_) => vec![],
        Type::Primitive(primitive) => vec![INTERNED_PRIMITIVES
            .get(primitive.as_str())
            .expect(&format!("primitive {primitive} wasn't interned"))],
        Type::FunctionPointer(_) => vec![],
        Type::Tuple(types) => types
            .into_iter()
            .flat_map(|type_| get_resolved_paths(krate, type_))
            .collect(),
        Type::Slice(type_) => get_resolved_paths(krate, type_),
        Type::Array { type_, .. } => get_resolved_paths(krate, type_),
        Type::ImplTrait(_) => vec![],
        Type::Infer => vec![],
        Type::RawPointer { type_, .. } => get_resolved_paths(krate, type_),
        Type::BorrowedRef { type_, .. } => get_resolved_paths(krate, type_),
        Type::QualifiedPath { .. } => vec![],
    }
}

/// Maps from primitives as found in [`rustdoc_types::Type::Primitive`] to paths `core::primitive::$ty`
static INTERNED_PRIMITIVES: once_cell::sync::Lazy<HashMap<&'static str, Vec<String>>> =
    once_cell::sync::Lazy::new(|| {
        let mut primitives = HashMap::new();
        macro_rules! insert_primitive {
            ($($ty:ty),* $(,)?) => {
                $(
                    primitives.insert(
                        stringify!($ty),
                        vec![
                            String::from("core"),
                            String::from("primitive"),
                            String::from(stringify!($ty)),
                        ]
                    );
                )*
            };
        }
        insert_primitive!(bool, char);
        insert_primitive!(f32, f64);
        insert_primitive!(i8, i16, i32, i64, i128, isize);
        insert_primitive!(u8, u16, u32, u64, u128, usize,);
        primitives
    });
