extern crate proc_macro;

//use checked_decimal_macro::num_traits::ToPrimitive;
use proc_macro::TokenStream;
use quote::quote;

use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct MacroInput {
    inner_type: syn::Type,
    len: u8,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner_type = input.parse::<syn::Type>()?;
        let _comma = input.parse::<syn::token::Comma>()?;
        let len = input.parse::<syn::LitInt>()?;

        Ok(MacroInput {
            inner_type,
            len: len.base10_parse().unwrap(),
        })
    }
}

#[proc_macro_attribute]
pub fn fixed_vector(args: TokenStream, item: TokenStream) -> TokenStream {
    let k = item.clone();
    let item_struct = parse_macro_input!(k as syn::ItemStruct);
    let struct_name = item_struct.ident;

    let args = syn::parse_macro_input!(args as MacroInput);
    let array_type = args.inner_type.clone();
    let array_length = args.len as usize;

    let mut stream = proc_macro::TokenStream::new();

    stream.extend(TokenStream::from(quote!(
        impl Default for #struct_name {
            fn default() -> #struct_name{
                #struct_name {
                    head: 0,
                    elements: [(); #array_length].map(|_| #array_type::default()),
                }
            }
        }

        impl #struct_name {
            pub fn iter<'a>(&'a self) -> Option<Iter<'a, #array_type>> {
                if self.head == 0 {
                    return None;
                }

                let range = ..self.head_usize();

                Some(self.elements.get(range)?.iter())
            }

            pub fn iter_mut<'a>(&'a mut self) -> Option<IterMut<'a, #array_type>> {
                if self.head == 0 {
                    return None;
                }

                let range = ..self.head_usize();

                Some(self.elements.get_mut(range)?.iter_mut())
            }

            pub fn find_mut(&mut self, search: &#array_type) -> Option<&mut #array_type> {
                if let Some(mut iter) = self.iter_mut() {
                    return iter.find(|el| *search == **el);
                }

                None
            }

            pub fn enumerate_find_mut(&mut self, search: &#array_type) -> Option<(usize, &mut #array_type)> {
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
                id < #array_length
            }

            /// returns immutable element under the index, does not check if it is before head,
            /// only check if it in array allocated area
            pub fn get(&self, id: usize) -> Option<&#array_type> {
                if self.index_in_capacity(id) {
                    self.elements.get(id)
                } else {
                    None
                }
            }

            /// returns mutable element under the index, does not check if it is before head,
            /// only check if it in array allocated area
            pub fn get_mut(&mut self, id: usize) -> Option<&mut #array_type> {
                if self.index_in_capacity(id) {
                    self.elements.get_mut(id)
                } else {
                    None
                }
            }

            /// returns mutable element under the index,
            /// check if it is in initialized range
            pub fn get_mut_checked(&mut self, id: usize) -> Option<&mut #array_type> {
                if self.index_before_head(id) {
                    self.elements.get_mut(id)
                } else {
                    None
                }
            }

            /// returns immutable mutable element under the index,
            /// check if it is in initialized range
            pub fn get_checked(&self, id: usize) -> Option<&#array_type> {
                if self.index_before_head(id) {
                    self.elements.get(id)
                } else {
                    None
                }
            }

            pub fn add(&mut self, el: #array_type) -> Result<(), ()> {
                let head = self.head_usize();

                if !self.index_in_capacity(head) {
                    return Err(());
                }

                *self.get_mut(head).ok_or(())? = el;
                self.head += 1;

                Ok(())
            }

            pub fn remove(&mut self) -> Option<&#array_type> {
                if self.head == 0 {
                    return None;
                }

                self.head -= 1;
                self.get(self.head_usize())
            }

            pub fn last_mut(&mut self) -> Option<&mut #array_type> {
                if self.head > 0 {
                    self.get_mut(self.head_usize() - 1)
                } else {
                    None
                }
            }

            pub fn last(&self) -> Option<&#array_type> {
                if self.head > 0 {
                    self.get(self.head_usize() - 1)
                } else {
                    None
                }
        }
    }
    )));

    stream.extend(item.clone());

    stream
}
