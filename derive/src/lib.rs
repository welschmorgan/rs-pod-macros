#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
  punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed, Ident, Meta,
  PathArguments, Type,
};

/// Allows derivation of a builder pattern on any struct
///
/// # Examples
///
/// ```rust,no-run
/// use crate::Builder;
///
/// #[derive(Builder)]
/// struct Data {
///   field: usize
/// }
///
/// fn main() {
///   let data = Data::builder().with_field(42).build();
///   assert_eq!(data, Data { field: 42 });
/// }
/// ```
#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(input as DeriveInput);

  let in_name = input.ident;
  let fname = format!("{}Builder", in_name);
  let builder_ty = syn::Ident::new(&fname, in_name.span());

  let orig_fields = match validate_struct(&input.data) {
    Ok(fields) => fields,
    Err(e) => return e.into(),
  };

  let field_accessors = orig_fields
    .named
    .iter()
    .map(|f| {
      let field_name = f.ident.clone().unwrap();
      let field_ty = f.ty.clone();
      let with_func_name = Ident::new(&format!("with_{}", field_name.clone()), f.span());
      // let get_func_name = Ident::new(&format!("get_{}", field_name.clone()), f.span());
      let set_func_name = Ident::new(&format!("set_{}", field_name.clone()), f.span());
      let ref_func_name = Ident::new(&format!("{}", field_name.clone()), f.span());
      let ref_mut_func_name = Ident::new(&format!("{}_mut", field_name.clone()), f.span());
      let unwrapped = if let Type::Path(path) = &field_ty {
        path
          .path
          .segments
          .iter()
          .find_map(|seg| match &seg.arguments {
            PathArguments::AngleBracketed(args) => args.args.first(),
            _ => None,
          })
      } else {
        None
      };
      let field_ty = match unwrapped {
        Some(ty) => quote! {#ty},
        None => quote! {#field_ty},
      };
      quote! {
        pub fn #with_func_name(mut self, v: #field_ty) -> Self {
          self.#field_name = Some(v);
          self
        }

        pub fn #ref_func_name(&self) -> Option<&#field_ty> {
          self.#field_name.as_ref()
        }

        pub fn #ref_mut_func_name(&mut self) -> &mut Option<#field_ty> {
          &mut self.#field_name
        }

        pub fn #set_func_name(&mut self, v: #field_ty) -> &mut Self {
          self.#field_name = Some(v);
          self
        }
      }
    })
    .collect::<proc_macro2::TokenStream>();
  let mut field_values: HashMap<Ident, proc_macro2::TokenStream> = HashMap::from_iter(
    orig_fields
      .named
      .iter()
      .flat_map(|f| {
        f.attrs.iter().find_map(move |attr| {
          if attr.path().is_ident("builder") {
            let nested = attr
              .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
              .unwrap();
            for meta in nested {
              match meta {
                Meta::NameValue(meta_name_value) => {
                  if meta_name_value.path.is_ident("default") {
                    println!(
                      "[{}] {} = {}",
                      f.ident.clone().to_token_stream().to_string(),
                      meta_name_value.path.to_token_stream().to_string(),
                      meta_name_value.value.to_token_stream().to_string()
                    );
                    let field_name = f.ident.clone().unwrap();
                    let field_value = meta_name_value.value.to_token_stream();
                    if is_option(&f.ty) {
                      return Some((
                        field_name.clone(),
                        quote! {
                          Some(self.#field_name.unwrap_or_else(|| #field_value))
                        },
                      ));
                    } else {
                      return Some((
                        field_name.clone(),
                        quote! {
                          self.#field_name.unwrap_or_else(|| #field_value)
                        },
                      ));
                    }
                  }
                }
                _ => {}
              }
            }
          }
          None
        })
      })
      .collect::<Vec<_>>(),
  );

  for orig_field in &orig_fields.named {
    let field_name = orig_field.ident.clone().unwrap();
    if !field_values.contains_key(&field_name) {
      field_values.insert(
        field_name.clone(),
        quote! {
          self.#field_name.unwrap_or_default()
        },
      );
    }
  }

  let new_fields = orig_fields
    .named
    .iter()
    .map(|field| {
      let field_name = field.ident.clone().unwrap();
      let field_vis = field.vis.clone();
      let field_ty = field.ty.clone();
      if is_option(&field_ty) {
        quote! {
          #field_vis #field_name: #field_ty,
        }
      } else {
        quote! {
          #field_vis #field_name: Option<#field_ty>,
        }
      }
    })
    .collect::<proc_macro2::TokenStream>();

  let builder_ctor: proc_macro2::TokenStream = orig_fields
    .named
    .iter()
    .map(|field| {
      let field_name = field.ident.clone().unwrap();
      quote! {
        #field_name: Default::default(),
      }
    })
    .collect();
  let builder_ctor: proc_macro2::TokenStream = quote! {
    #builder_ty {
      #builder_ctor
    }
  };

  let orig_ctor: proc_macro2::TokenStream = orig_fields
    .named
    .iter()
    .map(|field| {
      let field_name = field.ident.clone().unwrap();
      let field_value = &field_values[&field_name];
      quote! {
          #field_name: #field_value,
      }
    })
    .collect();
  let orig_ctor: proc_macro2::TokenStream = quote! {
    #in_name {
      #orig_ctor
    }
  };

  // Build the output, possibly using quasi-quotation
  let expanded = quote! {
        struct #builder_ty {
          #new_fields
        }

        impl Default for #builder_ty {
          fn default() -> Self {
            #builder_ctor
          }
        }

        impl pod_internal::Builder for #in_name {
          type Target = #builder_ty;

          fn builder() -> Self::Target {
            Self::Target::default()
          }
        }

        impl #builder_ty {
          #field_accessors

          pub fn build(mut self) -> #in_name {
            #orig_ctor
          }
        }
  };

  // Hand the output tokens back to the compiler
  TokenStream::from(expanded)
}

#[proc_macro_derive(Getters, attributes(getters))]
pub fn getters(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(input as DeriveInput);

  let in_name = input.ident;

  let orig_fields = match validate_struct(&input.data) {
    Ok(fields) => fields,
    Err(e) => return e.into(),
  };

  let mut field_skips: Vec<Ident> = vec![];
  for field in &orig_fields.named {
    for attr in &field.attrs {
      if attr.path().is_ident("getters") {
        let nested = attr
          .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
          .unwrap();
        for meta in nested {
          match meta {
            Meta::NameValue(meta_name_value) => {
              if meta_name_value.path.is_ident("skip") {
                return quote_spanned! {
                  attr.path().span() => compile_error!("`skip` attribute on `Getters` derive macro cannot have a value")
                }.into();
              }
            }
            Meta::Path(path) => {
              if path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
            Meta::List(list) => {
              if list.path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
          }
        }
      }
    }
  }
  let field_accessors = orig_fields
    .named
    .iter()
    .map(|f| {
      let field_name = f.ident.clone().unwrap();
      let field_ty = f.ty.clone();
      let ref_func_name = Ident::new(&format!("{}", field_name.clone()), f.span());
      let unwrapped = if let Type::Path(path) = &field_ty {
        path
          .path
          .segments
          .iter()
          .find_map(|seg| match &seg.arguments {
            PathArguments::AngleBracketed(args) => args.args.first(),
            _ => None,
          })
      } else {
        None
      };
      let unwrapped_field_ty = match unwrapped {
        Some(ty) => quote! {#ty},
        None => quote! {#field_ty},
      };

      if !field_skips.contains(&field_name) {
        if is_option(&field_ty) {
          quote! {
            pub fn #ref_func_name(&self) -> Option<&#unwrapped_field_ty> {
              self.#field_name.as_ref()
            }
          }
        } else {
          quote! {
            pub fn #ref_func_name(&self) -> &#field_ty {
              &self.#field_name
            }
          }
        }
      } else {
        quote! {}
      }
    })
    .collect::<proc_macro2::TokenStream>();

  // Build the output, possibly using quasi-quotation
  let expanded = quote! {
      impl #in_name {
        #field_accessors
      }
  };

  // Hand the output tokens back to the compiler
  TokenStream::from(expanded)
}

#[proc_macro_derive(Setters, attributes(setters))]
pub fn setters(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(input as DeriveInput);

  let in_name = input.ident;

  let orig_fields = match validate_struct(&input.data) {
    Ok(fields) => fields,
    Err(e) => return e.into(),
  };

  let mut field_skips: Vec<Ident> = vec![];
  for field in &orig_fields.named {
    for attr in &field.attrs {
      if attr.path().is_ident("setters") {
        let nested = attr
          .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
          .unwrap();
        for meta in nested {
          match meta {
            Meta::NameValue(meta_name_value) => {
              if meta_name_value.path.is_ident("skip") {
                return quote_spanned! {
                  attr.path().span() => compile_error!("`skip` attribute on `Setters` derive macro cannot have a value")
                }.into();
              }
            }
            Meta::Path(path) => {
              if path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
            Meta::List(list) => {
              if list.path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
          }
        }
      }
    }
  }

  let field_accessors = orig_fields
    .named
    .iter()
    .map(|f| {
      let field_name = f.ident.clone().unwrap();
      let field_ty = f.ty.clone();
      let ref_mut_func_name = Ident::new(&format!("{}_mut", field_name.clone()), f.span());
      let set_func_name = Ident::new(&format!("set_{}", field_name.clone()), f.span());
      let with_func_name = Ident::new(&format!("with_{}", field_name.clone()), f.span());
      if !field_skips.contains(&field_name) {
        if is_option(&field_ty) {
          quote! {
            pub fn #ref_mut_func_name(&mut self) -> &mut #field_ty {
              &mut self.#field_name
            }

            pub fn #set_func_name(&mut self, v: #field_ty) -> &mut Self {
              self.#field_name = v;
              self
            }

            pub fn #with_func_name(mut self, v: #field_ty) -> Self {
              self.#field_name = v;
              self
            }
          }
        } else {
          quote! {
            pub fn #ref_mut_func_name(&self) -> &#field_ty {
              &self.#field_name
            }

            pub fn #set_func_name(&mut self, v: #field_ty) -> &mut Self {
              self.#field_name = v;
              self
            }

            pub fn #with_func_name(mut self, v: #field_ty) -> Self {
              self.#field_name = v;
              self
            }
          }
        }
      } else {
        quote! {}
      }
    })
    .collect::<proc_macro2::TokenStream>();

  // Build the output, possibly using quasi-quotation
  let expanded = quote! {
      impl #in_name {
        #field_accessors
      }
  };

  // Hand the output tokens back to the compiler
  TokenStream::from(expanded)
}

#[proc_macro_derive(Fields, attributes(fields))]
pub fn fields(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(input as DeriveInput);

  let in_name = input.ident;

  let orig_fields = match validate_struct(&input.data) {
    Ok(fields) => fields,
    Err(e) => return e.into(),
  };

  let mut field_skips: Vec<Ident> = vec![];
  for field in &orig_fields.named {
    for attr in &field.attrs {
      if attr.path().is_ident("fields") {
        let nested = attr
          .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
          .unwrap();
        for meta in nested {
          match meta {
            Meta::NameValue(meta_name_value) => {
              if meta_name_value.path.is_ident("skip") {
                return quote_spanned! {
                  attr.path().span() => compile_error!("`skip` attribute on `Fields` derive macro cannot have a value")
                }.into();
              }
            }
            Meta::Path(path) => {
              if path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
            Meta::List(list) => {
              if list.path.is_ident("skip") {
                let field_name = field.ident.clone().unwrap();
                field_skips.push(field_name.clone());
              }
            }
          }
        }
      }
    }
  }
  let field_accessors = orig_fields
    .named
    .iter()
    .map(|f| {
      let field_name = f.ident.clone().unwrap();
      let field_ty = f.ty.clone();
      let ref_func_name = Ident::new(&format!("{}", field_name.clone()), f.span());
      let ref_mut_func_name = Ident::new(&format!("{}_mut", field_name.clone()), f.span());
      let set_func_name = Ident::new(&format!("set_{}", field_name.clone()), f.span());
      let with_func_name = Ident::new(&format!("with_{}", field_name.clone()), f.span());
      let unwrapped = if let Type::Path(path) = &field_ty {
        path
          .path
          .segments
          .iter()
          .find_map(|seg| match &seg.arguments {
            PathArguments::AngleBracketed(args) => args.args.first(),
            _ => None,
          })
      } else {
        None
      };
      let unwrapped_field_ty = match unwrapped {
        Some(ty) => quote! {#ty},
        None => quote! {#field_ty},
      };

      if !field_skips.contains(&field_name) {
        if is_option(&field_ty) {
          quote! {
            #[doc = concat!("Return the `", stringify!(#field_name), "` field as a mutable reference.")]
            pub fn #ref_mut_func_name(&mut self) -> &mut #field_ty {
              &mut self.#field_name
            }

            #[doc = concat!("Define the `", stringify!(#field_name), "` field.")]
            pub fn #set_func_name(&mut self, v: #field_ty) -> &mut Self {
              self.#field_name = v;
              self
            }

            #[doc = concat!("Define the `", stringify!(#field_name), "` field.")]
            pub fn #with_func_name(mut self, v: #field_ty) -> Self {
              self.#field_name = v;
              self
            }

            #[doc = concat!("Return the `", stringify!(#field_name), "` field.")]
            pub fn #ref_func_name(&self) -> Option<&#unwrapped_field_ty> {
              self.#field_name.as_ref()
            }
          }
        } else {
          quote! {
            #[doc = concat!("Retrieve the `", stringify!(#field_name), "` field as a mutable reference.")]
            pub fn #ref_mut_func_name(&mut self) -> &mut #field_ty {
              &mut self.#field_name
            }

            #[doc = concat!("Define the `", stringify!(#field_name), "` field.")]
            pub fn #set_func_name(&mut self, v: #field_ty) -> &mut Self {
              self.#field_name = v;
              self
            }

            #[doc = concat!("Define the `", stringify!(#field_name), "` field.")]
            pub fn #with_func_name(mut self, v: #field_ty) -> Self {
              self.#field_name = v;
              self
            }

            #[doc = concat!("Retrieve the `", stringify!(#field_name), "` field as a reference.")]
            pub fn #ref_func_name(&self) -> &#field_ty {
              &self.#field_name
            }
          }
        }
      } else {
        quote!{}
      }
    })
    .collect::<proc_macro2::TokenStream>();

  // Build the output, possibly using quasi-quotation
  let expanded = quote! {
      impl #in_name {
        #field_accessors
      }
  };

  // Hand the output tokens back to the compiler
  TokenStream::from(expanded)
}

fn is_option(ty: &Type) -> bool {
  match ty {
    Type::Path(path) if path.qself.is_none() => path
      .path
      .segments
      .iter()
      .find(|seg| {
        seg
          .ident
          .span()
          .source_text()
          .unwrap_or_default()
          .eq("Option")
      })
      .is_some(),
    _ => false,
  }
}
fn validate_struct(data: &Data) -> Result<&FieldsNamed, proc_macro2::TokenStream> {
  Ok(match data {
    Data::Struct(s) => match &s.fields {
      Fields::Named(fields) => fields,
      Fields::Unit => {
        return Err(quote_spanned! {
          s.struct_token.span() =>
            compile_error!("Builder pattern only available for named fields");
        })
      }

      Fields::Unnamed(u) => {
        return Err(quote_spanned! {
          u.paren_token.span =>
          compile_error!("Builder pattern only available for named fields");
        })
      }
    },
    Data::Enum(e) => {
      return Err(quote_spanned! {
          e.enum_token.span =>
          compile_error!("Builder pattern only available for Struct");
      })
    }
    Data::Union(u) => {
      return Err(quote_spanned! {
          u.union_token.span =>
          compile_error!("Builder pattern only available for Struct");
      })
    }
  })
}

#[proc_macro_derive(Ctor, attributes(ctor))]
pub fn ctor(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(input as DeriveInput);

  let in_ty = input.ident;

  let orig_fields = match validate_struct(&input.data) {
    Ok(fields) => fields,
    Err(e) => return e.into(),
  };

  let field_skips: HashMap<Ident, proc_macro2::TokenStream> = HashMap::from_iter(
    orig_fields
      .named
      .iter()
      .flat_map(|f| {
        f.attrs.iter().find_map(move |attr| {
          if attr.path().is_ident("ctor") {
            let nested = attr
              .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
              .unwrap();
            let field_name = f.ident.clone().unwrap();
            for meta in nested {
              match meta {
                Meta::NameValue(meta_name_value) => {
                  if meta_name_value.path.is_ident("skip") {
                    println!(
                      "[{}] {} = {}",
                      f.ident.clone().to_token_stream().to_string(),
                      meta_name_value.path.to_token_stream().to_string(),
                      meta_name_value.value.to_token_stream().to_string()
                    );
                    let field_value = meta_name_value.value.to_token_stream();
                    return Some((
                      field_name.clone(),
                      quote! {
                        #field_value
                      },
                    ));
                  }
                }
                Meta::Path(path) => {
                  if path.is_ident("skip") {
                    return Some((field_name.clone(), quote_spanned! {
                      path.span() => compile_error!("`skip` attribute on `Ctor` derive macro must have a value: the default value of the skipped field")
                    }))
                  }
                }
                Meta::List(list) => {
                  if list.path.is_ident("skip") {
                    return Some((field_name.clone(), quote_spanned! {
                      list.span() => compile_error!("`skip` attribute on `Ctor` derive macro must have a value: the default value of the skipped field")
                    }))
                  }
                }
                _ => {}
              }
            }
          }
          None
        })
      })
      .collect::<Vec<_>>(),
  );

  let orig_ctor_params: proc_macro2::TokenStream = orig_fields
    .named
    .iter()
    .map(|field| {
      let field_name = field.ident.as_ref().unwrap();
      let field_ty = &field.ty;
      if !field_skips.contains_key(field_name) {
        quote! {
            #field_name: #field_ty,
        }
      } else {
        quote! {}
      }
    })
    .collect();

  let orig_ctor: proc_macro2::TokenStream = orig_fields
    .named
    .iter()
    .map(|field| {
      let field_name = field.ident.clone().unwrap();
      if let Some(skipped_field) = field_skips.get(&field_name) {
        quote! {#field_name: #skipped_field,}
      } else {
        quote! {#field_name,}
      }
    })
    .collect();

  // Build the output, possibly using quasi-quotation
  let expanded = quote! {
    impl #in_ty {
      pub fn new(#orig_ctor_params) -> Self {
        Self {
          #orig_ctor
        }
      }
    }
  };

  // Hand the output tokens back to the compiler
  TokenStream::from(expanded)
}
