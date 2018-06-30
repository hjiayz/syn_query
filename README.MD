 #####implement Trait Syn::Visit
 #####jquery style
 [![Build Status](https://api.travis-ci.org/hjiayz/syn_query.svg?branch=master)](https://travis-ci.org/hjiayz/syn_query)
 [![Latest Version](https://img.shields.io/crates/v/syn_query.svg)](https://crates.io/crates/syn_query)
 [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/syn_query)
 [![Rustc Version 1.15+](https://img.shields.io/badge/rustc-1.15+-lightgray.svg)](https://blog.rust-lang.org/2017/02/02/Rust-1.15.html)

 ## Example
 ```rust
 extern crate proc_macro2;
 extern crate syn;
 extern crate syn_query;
 use proc_macro2::Span;
 use syn::{ExprStruct, FieldValue, Ident};
 use syn_query::Queryable;
 fn main() {
     let s = "Point { x: 1, y: 1 }";
     let st: ExprStruct = syn::parse_str(s).unwrap();
     let qr = st.query::<Ident>();
     assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
     assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
     assert_eq!(qr[1].data, Ident::new("x", Span::call_site()));
     assert_eq!(qr[1].path, vec![2i64, 0i64, 0i64]);
     assert_eq!(qr[2].data, Ident::new("y", Span::call_site()));
     assert_eq!(qr[2].path, vec![3i64, 0i64, 0i64]);
 }
 ```
