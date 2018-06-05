extern crate proc_macro2;
extern crate syn;
extern crate syn_query;
use proc_macro2::Span;
use syn::{ExprStruct, FieldValue, Ident};
use syn_query::Queryable;

#[test]
fn example() {
    let s = "Point { x: 1, y: 1 }";
    let st: ExprStruct = syn::parse_str(s).unwrap();
    let qr = st.query::<Ident>();
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
    assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
    assert_eq!(qr[1].data, Ident::new("x", Span::call_site()));
    assert_eq!(qr[1].path, vec![2i64, 0i64, 0i64]);
    assert_eq!(qr[2].data, Ident::new("y", Span::call_site()));
    assert_eq!(qr[2].path, vec![3i64, 0i64, 0i64]);
    let qr = st.query::<FieldValue>()
        .filter(|x| x.path[0] == 3)
        .query::<Ident>();
    assert_eq!(qr[0].data, Ident::new("y", Span::call_site()));
    assert_eq!(qr[0].path, vec![3i64, 0i64, 0i64]);
    let qr = st.children::<syn::Path>()
        .children::<syn::PathSegment>()
        .children::<Ident>();
    assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
    assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
    let qr = st.children::<syn::Path>().children::<Ident>();
    assert_eq!(qr.len(), 0);
}
