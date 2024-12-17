// Copyright (C) 2024 Ethan Uppal. See the LICENSE file for license information
// about this file.

use proc_macro::TokenStream;
use quote::quote;

fn impl_num_type(
    struct_visibility: syn::Visibility,
    struct_name: syn::Ident,
    inner_type: &syn::Type,
) -> proc_macro2::TokenStream {
    let syn::Type::Path(syn::TypePath {
        qself: None,
        path: type_path,
    }) = inner_type
    else {
        return syn::Error::new_spanned(
            inner_type,
            "Inner type should be a primitive integer.",
        )
        .into_compile_error();
    };

    let Some(type_identifier) = type_path.get_ident() else {
        return syn::Error::new_spanned(
            type_path,
            "Inner type should be a primitive integer.",
        )
        .into_compile_error();
    };

    let signedness_impl = match type_identifier.to_string().as_str() {
        "u8" | "u16" | "u32" | "u64" => quote! {
            impl num_traits::Unsigned for #struct_name {}
        },
        "i8" | "i16" | "i32" | "i64" => quote! {
            impl core::ops::Neg for #struct_name {
                type Output = Self;

                fn neg(self ) -> Self::Output {
                    Self(self.0.neg())
                }
            }

            impl num_traits::Signed for #struct_name {
                fn abs(&self) -> Self {
                    Self(self.0.abs())
                }

                fn abs_sub(&self, other: &Self) -> Self {
                    Self(self.0.abs_sub(&other.0))
                }

                fn signum(&self) -> Self {
                    Self(self.0.signum())
                }

                fn is_positive(&self) -> bool {
                    self.0.is_positive()
                }

                fn is_negative(&self) -> bool {
                    self.0.is_negative()
                }
            }
        },
        _ => {
            return syn::Error::new_spanned(
                type_path,
                "Inner type should be a primitive integer.",
            )
            .into_compile_error();
        }
    };

    quote! {
        impl core::ops::Add for #struct_name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0.add(rhs.0))
            }
        }

        impl num_traits::Zero for #struct_name {
            fn zero() -> Self {
                Self(#inner_type::zero())
            }

            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }
        }

        impl core::ops::Mul for #struct_name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0.mul(rhs.0))
            }
        }

        impl num_traits::One for #struct_name {
            fn one() -> Self {
                Self(#inner_type::one())
            }
        }

        impl core::ops::Sub for #struct_name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0.sub(rhs.0))
            }
        }

        impl core::ops::Div for #struct_name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0.div(rhs.0))
            }
        }

        impl core::ops::Rem for #struct_name {
            type Output = Self;

            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0.rem(rhs.0))
            }
        }

        impl num_traits::Num for #struct_name {
            type FromStrRadixErr = <#inner_type as num_traits::Num>::FromStrRadixErr;

            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(Self(#inner_type::from_str_radix(str, radix)?))
            }
        }

        #signedness_impl

        impl From<#inner_type> for #struct_name {
            fn from(value: #inner_type) -> Self {
                Self(value)
            }
        }

        impl From<#struct_name> for #inner_type {
            fn from(value: #struct_name) -> Self {
                value.0
            }
        }

        impl #struct_name {
            #struct_visibility const MIN: Self = Self(#inner_type::MIN);
            #struct_visibility const MAX: Self = Self(#inner_type::MAX);
            #struct_visibility const BITS: u32 = #inner_type::BITS;
        }
    }
}

#[proc_macro_attribute]
pub fn num_type(_args: TokenStream, input: TokenStream) -> TokenStream {
    mod kw {
        syn::custom_keyword!(transparent);
    }

    let input_item = syn::parse_macro_input!(input as syn::DeriveInput);
    let input_item_cloned = input_item.clone();

    let data_struct = match input_item.data {
        syn::Data::Struct(data_struct) => data_struct,
        syn::Data::Enum(syn::DataEnum {
            enum_token: syn::token::Enum { span },
            ..
        })
        | syn::Data::Union(syn::DataUnion {
            union_token: syn::token::Union { span },
            ..
        }) => {
            return syn::Error::new(span, "Item must be an `struct`.")
                .into_compile_error()
                .into();
        }
    };

    if input_item.attrs.iter().any(|attr| {
        matches!(attr.style, syn::AttrStyle::Outer)
            && attr
                .path()
                .is_ident("repr")
                .then(|| attr.parse_args::<kw::transparent>())
                .is_some()
    }) {
        let attributes = input_item.attrs;
        return syn::Error::new_spanned(
            quote! { #(#attributes)* },
            "Missing `#[repr(transparent)]` attribute on `struct`.",
        )
        .into_compile_error()
        .into();
    }

    let syn::Fields::Unnamed(inner) = data_struct.fields else {
        return syn::Error::new_spanned(
            data_struct.fields,
            "Item must be a tuple `struct` with one field.",
        )
        .into_compile_error()
        .into();
    };

    let fields = inner.unnamed.into_iter().collect::<Vec<_>>();
    let [syn::Field { ty: inner_type, .. }] = &fields[..] else {
        return syn::Error::new_spanned(
            quote! { #(#fields),* },
            "Item must have one field.",
        )
        .into_compile_error()
        .into();
    };

    let num_impl = impl_num_type(input_item.vis, input_item.ident, inner_type);

    quote! {
        #input_item_cloned

        #num_impl
    }
    .into()
}
