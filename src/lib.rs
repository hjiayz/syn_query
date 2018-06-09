//! implement Trait Syn::Visit
//! ## Example
//! ```rust
//! extern crate proc_macro2;
//! extern crate syn;
//! extern crate syn_query;
//! use syn_query::Queryable;
//! use proc_macro2::Span;
//! use syn::{ExprStruct, FieldValue, Ident};
//! fn main() {
//!     let s = "Point { x: 1, y: 1 }";
//!     let st: ExprStruct = syn::parse_str(s).unwrap();
//!     let qr = st.query::<Ident>();
//!     assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
//!     assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
//!     assert_eq!(qr[1].data, Ident::new("x", Span::call_site()));
//!     assert_eq!(qr[1].path, vec![2i64, 0i64, 0i64]);
//!     assert_eq!(qr[2].data, Ident::new("y", Span::call_site()));
//!     assert_eq!(qr[2].path, vec![3i64, 0i64, 0i64]);
//!     let qr = st.query::<FieldValue>()
//!         .filter(|x| x.path[0] == 3)
//!         .query::<Ident>();
//!     assert_eq!(qr[0].data, Ident::new("y", Span::call_site()));
//!     assert_eq!(qr[0].path, vec![3i64, 0i64, 0i64]);
//!     let qr = st.children::<syn::Path>().children::<syn::PathSegment>().children::<Ident>();
//!     assert_eq!(qr[0].data, Ident::new("Point", Span::call_site()));
//!     assert_eq!(qr[0].path, vec![0i64, 0i64, 0i64]);
//! }
//! ```

extern crate proc_macro2;
extern crate syn;
use proc_macro2::Span;
use std::any::Any;
use std::cmp::Ordering;
use std::ops::Index as OpsIndex;
use syn::visit::*;
use syn::*;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub data: T,
    pub path: Vec<i64>,
}

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.path == other.path
    }
}

impl<T> Ord for Node<T> {
    fn cmp(&self, other: &Node<T>) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl<T> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Node<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for Node<T> {}

#[derive(Debug)]
struct Query<T> {
    base: Vec<i64>,
    path: Vec<i64>,
    results: Vec<Node<T>>,
    deep: Option<usize>,
}
impl<T: Queryable> Query<T> {
    fn new(base: Vec<i64>, deep: Option<usize>) -> Query<T> {
        Query {
            base: base,
            path: Vec::new(),
            results: Vec::new(),
            deep: deep,
        }
    }
    fn mk_result(&mut self, i: T) {
        self.results.push(Node {
            path: self.base
                .clone()
                .into_iter()
                .chain(self.path.clone().into_iter())
                .collect(),
            data: i.clone(),
        });
    }
}

#[derive(Debug, Clone)]
pub struct QueryResult<T, R> {
    nodes: Vec<Node<T>>,
    root: R,
}

impl<T: Queryable, R: Queryable> OpsIndex<usize> for QueryResult<T, R> {
    type Output = Node<T>;
    fn index(&self, id: usize) -> &Node<T> {
        &(self.nodes[id])
    }
}

impl<T: Queryable, R: Queryable> QueryResult<T, R> {
    pub fn query<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut result = BTreeSet::new();
        for i in self.iter() {
            for j in i.data.visit(i.path.to_owned(), None) {
                result.insert(j);
            }
        }
        QueryResult::new(result.into_iter().collect(), self.root.to_owned())
    }
    pub fn find<U: Queryable>(&self) -> QueryResult<U, R> {
        self.query()
    }
    pub fn children<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut result = BTreeSet::new();
        for i in self.iter() {
            for j in i.data.visit(i.path.to_owned(), Some(1)) {
                result.insert(j);
            }
        }
        QueryResult::new(result.into_iter().collect(), self.root.to_owned())
    }
    pub fn new(result: Vec<Node<T>>, root: R) -> QueryResult<T, R> {
        QueryResult {
            nodes: result,
            root: root,
        }
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn iter(&self) -> std::slice::Iter<Node<T>> {
        self.nodes.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Node<T>> {
        self.nodes.iter_mut()
    }
    pub fn filter<P>(&self, predicate: P) -> QueryResult<T, R>
    where
        for<'r> P: FnMut(&'r Node<T>) -> bool,
    {
        QueryResult::new(
            self.to_owned().into_iter().filter(predicate).collect(),
            self.root.to_owned(),
        )
    }
    pub fn parents<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut path_list = BTreeSet::new();
        for item in self.iter() {
            let mut path = Vec::new();
            for path_node in item.path.iter() {
                path_list.insert(path.to_owned());
                path.push(path_node.to_owned());
            }
        }
        self.root
            .query()
            .filter(|node| path_list.contains(&node.path))
    }
    pub fn parent<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut path_list = BTreeSet::new();
        for item in self.iter() {
            let mut path = item.path.to_owned();
            if path.pop().is_some() {
                path_list.insert(path);
            }
        }
        self.root
            .query()
            .filter(|node| path_list.contains(&node.path))
    }
    pub fn prev<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut path_list = BTreeSet::new();
        for item in self.iter() {
            let mut path = item.path.to_owned();
            let mut will_insert = false;
            if let Some(last) = path.last_mut() {
                if *last > 0 {
                    *last -= 1;
                    will_insert = true;
                }
            }
            if will_insert {
                path_list.insert(path);
            };
        }
        self.root
            .query()
            .filter(|node| path_list.contains(&node.path))
    }
    pub fn prev_all<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for item in self.iter() {
            let mut path = item.path.to_owned();
            if let Some(last) = path.pop() {
                if map.get(&path).map_or(true, |value| (*value) > last) {
                    map.insert(path, last);
                }
            }
        }
        self.root.query().filter(|node| {
            let mut path = node.path.to_owned();
            if let Some(last) = path.pop() {
                let value = map.get(&path);
                return value.map_or(false, |value| (*value) > last);
            }
            false
        })
    }
    pub fn prev_until<U: Queryable, P>(&self, predicate: P) -> QueryResult<U, R>
    where
        for<'r> P: FnMut(&'r Node<U>) -> bool,
    {
        use std::collections::HashMap;
        let all: QueryResult<U, R> = self.prev_all();
        let unitl = all.filter(predicate);
        let mut map = HashMap::new();
        for item in unitl {
            let mut path = item.path;
            if let Some(last) = path.pop() {
                if map.get(&path).map_or(true, |value| (*value) > last) {
                    map.insert(path, last);
                }
            }
        }
        all.filter(|node| {
            let mut path = node.path.to_owned();
            if let Some(last) = path.pop() {
                return map.get(&path).map_or(true, |value| (*value) < last);
            }
            false
        })
    }
    pub fn next<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::BTreeSet;
        let mut path_list = BTreeSet::new();
        for item in self.to_owned().into_iter() {
            let mut path = item.path.to_owned();
            let mut will_insert = false;
            if let Some(last) = path.last_mut() {
                *last += 1;
                will_insert = true;
            }
            if will_insert {
                path_list.insert(path);
            };
        }
        self.root
            .query()
            .filter(|node| path_list.contains(&node.path))
    }
    pub fn next_all<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for item in self.iter() {
            let mut path = item.path.to_owned();
            if let Some(last) = path.pop() {
                if map.get(&path).map_or(true, |value| (*value) < last) {
                    map.insert(path, last);
                }
            }
        }
        self.root.query().filter(|node| {
            let mut path = node.path.to_owned();
            if let Some(last) = path.pop() {
                let value = map.get(&path);
                return value.map_or(false, |value| (*value) < last);
            }
            false
        })
    }
    pub fn next_until<U: Queryable, P>(&self, predicate: P) -> QueryResult<U, R>
    where
        for<'r> P: FnMut(&'r Node<U>) -> bool,
    {
        use std::collections::HashMap;
        let all: QueryResult<U, R> = self.next_all();
        let unitl = all.filter(predicate);
        let mut map = HashMap::new();
        for item in unitl {
            let mut path = item.path;
            if let Some(last) = path.pop() {
                if map.get(&path).map_or(true, |value| (*value) < last) {
                    map.insert(path, last);
                }
            }
        }
        all.filter(|node| {
            let mut path = node.path.to_owned();
            if let Some(last) = path.pop() {
                return map.get(&path).map_or(true, |value| (*value) > last);
            }
            false
        })
    }
    pub fn siblings<U: Queryable>(&self) -> QueryResult<U, R> {
        use std::collections::HashMap;
        let mut map = HashMap::<_, Option<i64>>::new();
        for node in self.nodes.to_owned().into_iter() {
            let mut path = node.path;
            if let Some(last) = path.pop().to_owned() {
                match map.get(&path).map(|item| *item) {
                    None => {
                        map.insert(path, Some(last.to_owned()));
                    }
                    Some(item) => {
                        if item.map_or(false, |item| item != last) {
                            map.insert(path, None);
                        }
                    }
                }
            }
        }
        self.root.find().filter(|node| {
            let mut path = node.path.to_owned();
            if let Some(last) = path.pop() {
                return map.get(&path)
                    .map_or(false, |value| value.map_or(true, |value| value != last));
            }
            false
        })
    }
    pub fn eq(&self, index: isize) -> Option<Node<T>> {
        let id = if index >= 0 {
            index as usize
        } else {
            let offset = -index as usize;
            if offset > self.len() {
                return None;
            }
            self.len() - offset
        };
        self.nodes.get(id).map(|node| node.to_owned())
    }
    pub fn first(&self) -> Option<Node<T>> {
        self.nodes.first().map(|node| node.to_owned())
    }
    pub fn last(&self) -> Option<Node<T>> {
        self.nodes.last().map(|node| node.to_owned())
    }
    pub fn map<B, F>(&self, f: F) -> Vec<B>
    where
        F: FnMut(Node<T>) -> B,
    {
        self.nodes.to_owned().into_iter().map(f).collect()
    }
    pub fn is<F>(&self, f: F) -> bool
    where
        for<'r> F: FnMut(&'r Node<T>) -> bool,
    {
        self.iter().any(f)
    }
    pub fn has(&self) -> bool {
        self.len() > 0
    }
    pub fn not<P>(&self, mut predicate: P) -> QueryResult<T, R>
    where
        for<'r> P: FnMut(&'r Node<T>) -> bool,
    {
        self.filter(|p: &Node<T>| !(predicate(p)))
    }
}

impl<T: Queryable, R: Queryable> IntoIterator for QueryResult<T, R> {
    type Item = Node<T>;
    type IntoIter = ::std::vec::IntoIter<Node<T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

pub trait Queryable: Sized + 'static + Clone {
    fn visit<U: Queryable>(&self, base: Vec<i64>, deep: Option<usize>) -> Vec<Node<U>>;
    fn query<U: Queryable>(&self) -> QueryResult<U, Self> {
        query::<_, _>(self.to_owned())
    }
    fn find<U: Queryable>(&self) -> QueryResult<U, Self> {
        query::<_, _>(self.to_owned())
    }
    fn children<U: Queryable>(&self) -> QueryResult<U, Self> {
        children::<_, _>(self.to_owned())
    }
}

macro_rules! build_visit {
    ($( $struct_name:ident:$fn_name:ident ),*) => (

        $(
            impl Queryable for $struct_name {
                fn visit<U: Queryable >(&self, base: Vec<i64>,deep:Option<usize>) -> Vec<Node<U>> {
                    let mut query = Query::new(base,deep);
                    query. $fn_name (self);
                    query.results
                }
            }
        )*

        impl<'ast,T:Queryable+Clone+Any> visit::Visit<'ast> for Query<T> {
            $(
                fn $fn_name(&mut self, i: &'ast $struct_name) {
                    let val=i as &Any;
                    if let Some(result) = val.downcast_ref::<T>() {
                        self.mk_result(result.to_owned());
                    }
                    if self.deep.is_none()||self.path.len()<self.deep.unwrap() {
                    self.path.push(0);
                    $fn_name(self, i);
                    self.path.pop();
                    }
                    if let Some(last) = self.path.last_mut() {
                        *last+=1
                    }
                }
            )*
        }
    )
}

build_visit!(
    Abi: visit_abi,
    AngleBracketedGenericArguments: visit_angle_bracketed_generic_arguments,
    ArgCaptured: visit_arg_captured,
    ArgSelf: visit_arg_self,
    ArgSelfRef: visit_arg_self_ref,
    Arm: visit_arm,
    AttrStyle: visit_attr_style,
    Attribute: visit_attribute,
    BareFnArg: visit_bare_fn_arg,
    BareFnArgName: visit_bare_fn_arg_name,
    BinOp: visit_bin_op,
    Binding: visit_binding,
    Block: visit_block,
    BoundLifetimes: visit_bound_lifetimes,
    ConstParam: visit_const_param,
    Data: visit_data,
    DataEnum: visit_data_enum,
    DataStruct: visit_data_struct,
    DataUnion: visit_data_union,
    DeriveInput: visit_derive_input,
    Expr: visit_expr,
    ExprArray: visit_expr_array,
    ExprAssign: visit_expr_assign,
    ExprAssignOp: visit_expr_assign_op,
    ExprBinary: visit_expr_binary,
    ExprBlock: visit_expr_block,
    ExprBox: visit_expr_box,
    ExprBreak: visit_expr_break,
    ExprCall: visit_expr_call,
    ExprCast: visit_expr_cast,
    ExprCatch: visit_expr_catch,
    ExprClosure: visit_expr_closure,
    ExprContinue: visit_expr_continue,
    ExprField: visit_expr_field,
    ExprForLoop: visit_expr_for_loop,
    ExprGroup: visit_expr_group,
    ExprIf: visit_expr_if,
    ExprIfLet: visit_expr_if_let,
    ExprInPlace: visit_expr_in_place,
    ExprIndex: visit_expr_index,
    ExprLit: visit_expr_lit,
    ExprLoop: visit_expr_loop,
    ExprMacro: visit_expr_macro,
    ExprMatch: visit_expr_match,
    ExprMethodCall: visit_expr_method_call,
    ExprParen: visit_expr_paren,
    ExprPath: visit_expr_path,
    ExprRange: visit_expr_range,
    ExprReference: visit_expr_reference,
    ExprRepeat: visit_expr_repeat,
    ExprReturn: visit_expr_return,
    ExprStruct: visit_expr_struct,
    ExprTry: visit_expr_try,
    ExprTuple: visit_expr_tuple,
    ExprType: visit_expr_type,
    ExprUnary: visit_expr_unary,
    ExprUnsafe: visit_expr_unsafe,
    ExprVerbatim: visit_expr_verbatim,
    ExprWhile: visit_expr_while,
    ExprWhileLet: visit_expr_while_let,
    ExprYield: visit_expr_yield,
    Field: visit_field,
    FieldPat: visit_field_pat,
    FieldValue: visit_field_value,
    Fields: visit_fields,
    FieldsNamed: visit_fields_named,
    FieldsUnnamed: visit_fields_unnamed,
    File: visit_file,
    FnArg: visit_fn_arg,
    FnDecl: visit_fn_decl,
    ForeignItem: visit_foreign_item,
    ForeignItemFn: visit_foreign_item_fn,
    ForeignItemStatic: visit_foreign_item_static,
    ForeignItemType: visit_foreign_item_type,
    ForeignItemVerbatim: visit_foreign_item_verbatim,
    GenericArgument: visit_generic_argument,
    GenericMethodArgument: visit_generic_method_argument,
    GenericParam: visit_generic_param,
    Generics: visit_generics,
    Ident: visit_ident,
    ImplItem: visit_impl_item,
    ImplItemConst: visit_impl_item_const,
    ImplItemMacro: visit_impl_item_macro,
    ImplItemMethod: visit_impl_item_method,
    ImplItemType: visit_impl_item_type,
    ImplItemVerbatim: visit_impl_item_verbatim,
    Index: visit_index,
    Item: visit_item,
    ItemConst: visit_item_const,
    ItemEnum: visit_item_enum,
    ItemExternCrate: visit_item_extern_crate,
    ItemFn: visit_item_fn,
    ItemForeignMod: visit_item_foreign_mod,
    ItemImpl: visit_item_impl,
    ItemMacro: visit_item_macro,
    ItemMacro2: visit_item_macro2,
    ItemMod: visit_item_mod,
    ItemStatic: visit_item_static,
    ItemStruct: visit_item_struct,
    ItemTrait: visit_item_trait,
    ItemType: visit_item_type,
    ItemUnion: visit_item_union,
    ItemUse: visit_item_use,
    ItemVerbatim: visit_item_verbatim,
    Label: visit_label,
    Lifetime: visit_lifetime,
    LifetimeDef: visit_lifetime_def,
    Lit: visit_lit,
    LitBool: visit_lit_bool,
    LitByte: visit_lit_byte,
    LitByteStr: visit_lit_byte_str,
    LitChar: visit_lit_char,
    LitFloat: visit_lit_float,
    LitInt: visit_lit_int,
    LitStr: visit_lit_str,
    LitVerbatim: visit_lit_verbatim,
    Local: visit_local,
    Macro: visit_macro,
    MacroDelimiter: visit_macro_delimiter,
    Member: visit_member,
    Meta: visit_meta,
    MetaList: visit_meta_list,
    MetaNameValue: visit_meta_name_value,
    MethodSig: visit_method_sig,
    MethodTurbofish: visit_method_turbofish,
    NestedMeta: visit_nested_meta,
    ParenthesizedGenericArguments: visit_parenthesized_generic_arguments,
    Pat: visit_pat,
    PatBox: visit_pat_box,
    PatIdent: visit_pat_ident,
    PatLit: visit_pat_lit,
    PatMacro: visit_pat_macro,
    PatPath: visit_pat_path,
    PatRange: visit_pat_range,
    PatRef: visit_pat_ref,
    PatSlice: visit_pat_slice,
    PatStruct: visit_pat_struct,
    PatTuple: visit_pat_tuple,
    PatTupleStruct: visit_pat_tuple_struct,
    PatVerbatim: visit_pat_verbatim,
    PatWild: visit_pat_wild,
    Path: visit_path,
    PathArguments: visit_path_arguments,
    PathSegment: visit_path_segment,
    PredicateEq: visit_predicate_eq,
    PredicateLifetime: visit_predicate_lifetime,
    PredicateType: visit_predicate_type,
    QSelf: visit_qself,
    RangeLimits: visit_range_limits,
    ReturnType: visit_return_type,
    Span: visit_span,
    Stmt: visit_stmt,
    TraitBound: visit_trait_bound,
    TraitBoundModifier: visit_trait_bound_modifier,
    TraitItem: visit_trait_item,
    TraitItemConst: visit_trait_item_const,
    TraitItemMacro: visit_trait_item_macro,
    TraitItemMethod: visit_trait_item_method,
    TraitItemType: visit_trait_item_type,
    TraitItemVerbatim: visit_trait_item_verbatim,
    Type: visit_type,
    TypeArray: visit_type_array,
    TypeBareFn: visit_type_bare_fn,
    TypeGroup: visit_type_group,
    TypeImplTrait: visit_type_impl_trait,
    TypeInfer: visit_type_infer,
    TypeMacro: visit_type_macro,
    TypeNever: visit_type_never,
    TypeParam: visit_type_param,
    TypeParamBound: visit_type_param_bound,
    TypeParen: visit_type_paren,
    TypePath: visit_type_path,
    TypePtr: visit_type_ptr,
    TypeReference: visit_type_reference,
    TypeSlice: visit_type_slice,
    TypeTraitObject: visit_type_trait_object,
    TypeTuple: visit_type_tuple,
    TypeVerbatim: visit_type_verbatim,
    UnOp: visit_un_op,
    UseGlob: visit_use_glob,
    UseGroup: visit_use_group,
    UseName: visit_use_name,
    UsePath: visit_use_path,
    UseRename: visit_use_rename,
    UseTree: visit_use_tree,
    Variant: visit_variant,
    VisCrate: visit_vis_crate,
    VisPublic: visit_vis_public,
    VisRestricted: visit_vis_restricted,
    Visibility: visit_visibility,
    WhereClause: visit_where_clause,
    WherePredicate: visit_where_predicate
);

pub fn query<T: Queryable, U: Queryable>(i: U) -> QueryResult<T, U> {
    QueryResult::new(i.visit(Vec::new(), None), i.to_owned())
}

pub fn find<T: Queryable, U: Queryable>(i: U) -> QueryResult<T, U> {
    query(i)
}

pub fn children<T: Queryable, U: Queryable>(i: U) -> QueryResult<T, U> {
    QueryResult::new(i.visit(Vec::new(), Some(1)), i.to_owned())
}
