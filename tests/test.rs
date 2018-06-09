extern crate proc_macro2;
extern crate syn;
extern crate syn_query;
use proc_macro2::Span;
use syn::{ExprStruct, FieldValue, Ident};
use syn_query::Queryable;

#[test]
fn find() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<Ident>();
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
    assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
    assert_eq!(qr[1].data, Ident::new("x", Span::call_site()));
    assert_eq!(qr[1].path, vec![2i64, 0i64, 0i64]);
    assert_eq!(qr[2].data, Ident::new("y", Span::call_site()));
    assert_eq!(qr[2].path, vec![3i64, 0i64, 0i64]);

    let qr = st.find::<FieldValue>()
        .filter(|x| x.path[0] == 3)
        .find::<Ident>();
    assert_eq!(qr[0].data, Ident::new("y", Span::call_site()));
    assert_eq!(qr[0].path, vec![3i64, 0i64, 0i64]);
}

#[test]
fn filter() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<FieldValue>()
        .filter(|x| x.path[0] == 3)
        .find::<Ident>();
    assert_eq!(qr[0].data, Ident::new("y", Span::call_site()));
    assert_eq!(qr[0].path, vec![3i64, 0i64, 0i64]);
}

#[test]
fn children() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.children::<syn::Path>()
        .children::<syn::PathSegment>()
        .children::<Ident>();
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
    assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);

    let qr = st.children::<syn::Path>().children::<Ident>();
    assert_eq!(qr.len(), 0);
}

#[test]
fn parent() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Path>().parents::<ExprStruct>();
    assert_eq!(qr.len(), 1);

    let qr = st.find::<syn::Path>().parents::<syn::Path>();
    assert_eq!(qr.len(), 0);

    let qr = st.find::<syn::Path>()
        .parents::<ExprStruct>()
        .children::<syn::Path>();
    assert_eq!(qr.len(), 1);

    let qr = st.find::<syn::Path>()
        .parents::<ExprStruct>()
        .children::<syn::Path>()
        .children::<syn::PathSegment>()
        .children::<Ident>();
    assert_eq!(qr.len(), 1);

    let qr = st.children::<Span>().parent::<ExprStruct>().find::<Ident>();
    assert_eq!(qr.len(), 3);
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
    let qr = st.children::<Span>()
        .parent::<ExprStruct>()
        .parent::<ExprStruct>();
    assert_eq!(qr.len(), 0);
}

#[test]
fn next() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<Span>().next::<syn::FieldValue>().find::<Ident>();
    assert_eq!(qr.len(), 1);
    assert_eq!(qr[0].data, Ident::new("x", Span::call_site()));

    let qr = st.find::<syn::Path>().next::<Span>();
    assert_eq!(qr.len(), 1);
    assert_eq!(format!("{:?}", qr[0].data), "Span");

    let qr = st.find::<Span>().next_all::<syn::FieldValue>();
    assert_eq!(qr.len(), 2);
    assert_eq!(
        qr[0].data.find::<Ident>()[0].data,
        Ident::new("x", Span::call_site())
    );

    let qr = st.find::<Span>()
        .next_until::<syn::FieldValue, _>(|node| node.data.find::<Ident>()[0].data == "y");
    assert_eq!(qr.len(), 1);
    assert_eq!(
        qr[0].data.find::<Ident>()[0].data,
        Ident::new("x", Span::call_site())
    );
}

#[test]
fn prev() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<FieldValue>()
        .prev::<syn::FieldValue>()
        .find::<Ident>();
    assert_eq!(qr.len(), 1);
    assert_eq!(qr[0].data, Ident::new("x", Span::call_site()));

    let qr = st.find::<Span>().prev::<syn::Path>().find::<Ident>();
    assert_eq!(qr.len(), 1);
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));

    let qr = st.find::<Span>().prev_all::<syn::Path>();
    assert_eq!(qr.len(), 1);

    let qr = st.find::<Span>()
        .prev_until::<syn::Path, _>(|node| node.data.find::<Ident>()[0].data == "Point");
    assert_eq!(qr.len(), 0);
}

#[test]
fn siblings() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Path>().siblings::<syn::FieldValue>();
    assert_eq!(qr.len(), 2);
    assert_eq!(qr.find::<Ident>()[1].data, "y");
}

#[test]
fn first() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().first();
    assert_eq!(qr.unwrap().data, "Point");
}

#[test]
fn last() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().last();
    assert_eq!(qr.unwrap().data, "y");
}

#[test]
fn eq() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().eq(-1);
    assert_eq!(qr.unwrap().data, "y");

    let qr = st.find::<syn::Ident>().eq(1);
    assert_eq!(qr.unwrap().data, "x");
}

#[test]
fn map() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().map(|node| node.data);
    assert_eq!(qr[0], "Point");
}

#[test]
fn is() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().is(|node| node.data == "Point");
    assert_eq!(qr, true);
}

#[test]
fn has() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().has();
    assert_eq!(qr, true);
}

#[test]
fn not() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();

    let qr = st.find::<syn::Ident>().not(|node| node.data == "Point");
    assert_eq!(qr.len(), 2);
    assert_eq!(qr[1].data, "y");
}
