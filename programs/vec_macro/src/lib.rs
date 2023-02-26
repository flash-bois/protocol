use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Expr, Generics, Ident, Lit, Token, Type, TypeArray, Visibility,
};

#[allow(dead_code)]
struct FixedArrayStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: syn::token::Brace,
    pub vis1: Visibility,
    pub head_ident: Ident,
    pub colon_token: Token![:],
    pub head_ty: Type,
    pub punctuation: Token![,],
    pub vis2: Visibility,
    pub arr_ident: Ident,
    pub colon_token2: Token![:],
    pub array: TypeArray,
    pub semi_token2: Option<Token![,]>,
}

impl Parse for FixedArrayStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let context;

        Ok(FixedArrayStruct {
            attrs,
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            generics: input.parse()?,
            brace_token: braced!(context in input),
            vis1: context.parse()?,
            head_ident: context.parse()?,
            colon_token: context.parse()?,
            head_ty: context.parse()?,
            punctuation: context.parse()?,
            vis2: context.parse()?,
            arr_ident: context.parse()?,
            colon_token2: context.parse()?,
            array: context.parse()?,
            semi_token2: context.parse()?,
        })
    }
}

#[proc_macro_derive(SafeArray)]
pub fn fixed_derive(tokens: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(tokens as FixedArrayStruct);
    let struct_name = parsed_struct.ident;

    let head_name = parsed_struct.head_ident;
    let arr_name = parsed_struct.arr_ident;
    let el_type = *parsed_struct.array.elem;

    let len: usize = match parsed_struct.array.len {
        Expr::Lit(ref l) => match l.lit {
            Lit::Int(ref int) => int.base10_parse().unwrap(),
            _ => panic!("cannot parse array length"),
        },
        _ => panic!("cannot parse array length"),
    };

    let mut stream = TokenStream::new();

    stream.extend(TokenStream::from(quote!(
        impl #struct_name {
            pub fn new() -> #struct_name {
                #struct_name {
                    #head_name: 0,
                    #arr_name: [(); #len].map(|_| #el_type::default()),
                }
            }

            pub fn iter<'a>(&'a self) -> Option<Iter<'a, #el_type>> {
                if self.head == 0 {
                    return None;
                }

                let range = ..self.head_usize();

                Some(self.elements.get(range)?.iter())
            }

            pub fn iter_mut<'a>(&'a mut self) -> Option<IterMut<'a, #el_type>> {
                if self.head == 0 {
                    return None;
                }

                let range = ..self.head_usize();

                Some(self.elements.get_mut(range)?.iter_mut())
            }

            pub fn find_mut(&mut self, search: &#el_type) -> Option<&mut #el_type> {
                if let Some(mut iter) = self.iter_mut() {
                    return iter.find(|el| *search == **el);
                }

                None
            }

            pub fn enumerate_find_mut(&mut self, search: &#el_type) -> Option<(usize, &mut #el_type)> {
                if let Some(iter) = self.iter_mut() {
                    return iter.enumerate().find(|(_id, pos)| *search == **pos);
                }

                None
            }

            pub fn delete(&mut self, id: usize) {
                // checks if id is before vector head
                assert!(self.index_before_head(id), "bad index");

                // move element that has to be delete to last position, shifting rest by -1
                // then it removes last position
                if let Some(iter) = self.iter_mut() {
                    iter.into_slice().get_mut(id..).unwrap().rotate_left(1);
                    self.remove();
                }
            }

            pub fn indexes(&self) -> Range<usize> {
                0..(self.head as usize)
            }

            // parse head u8 to usize
            pub fn head_usize(&self) -> usize {
                // unwrap because we assert if N can be fitted inside u8 on creation
                self.head.to_usize().unwrap()
            }

            /// checks if index is in useful range
            fn index_before_head(&self, id: usize) -> bool {
                self.head > 0 && id < self.head_usize()
            }

            /// checks if index is in allocated range
            fn index_in_capacity(&self, id: usize) -> bool {
                id < #len
            }

            /// returns immutable element under the index, does not check if it is before head,
            /// only check if it in array allocated area
            pub fn get(&self, id: usize) -> Option<&#el_type> {
                if self.index_in_capacity(id) {
                    self.elements.get(id)
                } else {
                    None
                }
            }

            /// returns mutable element under the index, does not check if it is before head,
            /// only check if it in array allocated area
            pub fn get_mut(&mut self, id: usize) -> Option<&mut #el_type> {
                if self.index_in_capacity(id) {
                    self.elements.get_mut(id)
                } else {
                    None
                }
            }

            /// returns mutable element under the index,
            /// check if it is in initialized range
            pub fn get_mut_checked(&mut self, id: usize) -> Option<&mut #el_type> {
                if self.index_before_head(id) {
                    self.elements.get_mut(id)
                } else {
                    None
                }
            }

            /// returns immutable mutable element under the index,
            /// check if it is in initialized range
            pub fn get_checked(&self, id: usize) -> Option<&#el_type> {
                if self.index_before_head(id) {
                    self.elements.get(id)
                } else {
                    None
                }
            }

            pub fn add(&mut self, el: #el_type) -> std::result::Result<(), ()> {
                let head = self.head_usize();

                if !self.index_in_capacity(head) {
                    return Err(());
                }

                *self.get_mut(head).ok_or(())? = el;
                self.head += 1;

                Ok(())
            }

            pub fn remove(&mut self) -> Option<&#el_type> {
                if self.head == 0 {
                    return None;
                }

                self.head -= 1;
                self.get(self.head_usize())
            }

            pub fn last_mut(&mut self) -> Option<&mut #el_type> {
                if self.head > 0 {
                    self.get_mut(self.head_usize() - 1)
                } else {
                    None
                }
            }

            pub fn last(&self) -> Option<&#el_type> {
                if self.head > 0 {
                    self.get(self.head_usize() - 1)
                } else {
                    None
                }
        }
        }

        impl Default for #struct_name {
            fn default() -> #struct_name{
                #struct_name::new()
            }
        }
    )));

    stream
}
