#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, parse::Parse, parse_macro_input};

fn extract_context_attr(attrs: &[Attribute]) -> Result<Option<syn::Type>, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("context") {
            return match &attr.meta {
                Meta::List(meta_list) => {
                    // Парсим содержимое внутри #[context(...)]
                    let parser = syn::parse::Parser::parse2;
                    let context_type: syn::Type =
                        parser(syn::Type::parse, meta_list.tokens.clone())?;
                    Ok(Some(context_type))
                }
                Meta::NameValue(name_value) => {
                    // Парсим #[context = "TypeName"]
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &name_value.value
                    {
                        let type_str = lit_str.value();
                        let context_type: syn::Type = syn::parse_str(&type_str)?;
                        Ok(Some(context_type))
                    } else {
                        Err(syn::Error::new_spanned(
                            &name_value.value,
                            "Expected string literal, e.g. #[context = \"MyAppEvent\"]",
                        ))
                    }
                }
                Meta::Path(_) => Err(syn::Error::new_spanned(
                    attr,
                    "Expected context type. Use #[context(TypeName)] or #[context = \"TypeName\"]",
                )),
            };
        }
    }
    Ok(None)
}

#[proc_macro_derive(WidgetEnum, attributes(context))]
pub fn widget_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Извлекаем атрибут #[context]
    let context_type = extract_context_attr(&input.attrs).expect("Missing attribute #[context]");

    let variants = if let Data::Enum(data_enum) = input.data {
        data_enum
            .variants
            .into_iter()
            .map(|v| {
                let vname = v.ident;
                match v.fields {
                    Fields::Unnamed(fields) => {
                        let inner = &fields.unnamed[0].ty;
                        (vname, inner.clone())
                    }
                    _ => panic!("WidgetEnum only supports tuple enums with one field"),
                }
            })
            .collect::<Vec<_>>()
    } else {
        panic!("WidgetEnum can only be derived for enums");
    };

    let get_element_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.get_element(id), }
    });

    let get_mut_element_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.get_mut_element(id), }
    });

    let id_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.id(), }
    });

    let anchor_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.anchor(), }
    });

    let desired_size_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.desired_size(), }
    });

    let as_any_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner, }
    });

    let as_any_mut_match = as_any_match.clone();

    let draw_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.draw(out), }
    });

    let layout_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.layout(bounds), }
    });

    let update_match = variants.iter().map(|(vname, _)| {
        quote! { #name::#vname(inner) => inner.update(ctx, sender), }
    });

    let expanded = quote! {
        impl WidgetQuery<WindowContext> for #name {
            fn get_element<QW: toolkit::widget::Widget<WindowContext>>(&self, id: &str) -> Option<&QW> {
                match self {
                    #(#get_element_match)*
                }
            }

            fn get_mut_element<QW: toolkit::widget::Widget<WindowContext>>(
                &mut self,
                id: &str,
            ) -> Option<&mut QW> {
                match self {
                    #(#get_mut_element_match)*
                }
            }

            fn id(&self) -> Option<&str> {
                match self {
                    #(#id_match)*
                }
            }

            fn as_any(&self) -> &dyn std::any::Any {
                match self {
                    #(#as_any_match)*
                }
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                match self {
                    #(#as_any_mut_match)*
                }
            }
        }

        impl toolkit::widget::Widget<#context_type> for #name {
            fn anchor(&self) -> toolkit::widget::Anchor {
                match self {
                    #(#anchor_match)*
                }
            }

            fn desired_size(&self) -> toolkit::widget::DesiredSize {
                match self {
                    #(#desired_size_match)*
                }
            }

            fn draw<'frame>(&'frame self, out: &mut toolkit::commands::CommandBuffer<'frame>) {
                match self {
                    #(#draw_match)*
                }
            }

            fn layout(&mut self, bounds: toolkit::types::Bounds) {
                match self {
                    #(#layout_match)*
                }
            }

            fn update(&mut self, ctx: &toolkit::widget::FrameContext, sender: &mut toolkit::widget::Sender<#context_type>) {
                match self {
                    #(#update_match)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Queryable, attributes(content))]
pub fn widget_query_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;

    // Извлекаем информацию о полях
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("WidgetQuery can only be derived for structs with named fields"),
        },
        _ => panic!("WidgetQuery can only be derived for structs"),
    };

    // Находим поле с атрибутом #[content]
    let content_field = fields.iter().find(|field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("content"))
    });

    // Генерируем реализацию в зависимости от наличия content поля
    let impl_block = if let Some(content_field) = content_field {
        let content_field_name = content_field.ident.as_ref();
        generate_with_content(name, generics, content_field_name)
    } else {
        generate_without_content(name, generics)
    };

    TokenStream::from(impl_block)
}

fn generate_with_content(
    name: &syn::Ident,
    generics: &syn::Generics,
    content_field_name: Option<&syn::Ident>,
) -> proc_macro2::TokenStream {
    let content_field_name = content_field_name
        .as_ref()
        .expect("Content field must have a name");

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics graphics::widget::Queryable for #name #ty_generics #where_clause {
            fn get_element_dyn(&self, id: &str) -> Option<&dyn std::any::Any> {
                if self.id.as_deref() == Some(id) {
                    return Some(self.as_any());
                }

                self.#content_field_name.get_element_dyn(id)
            }

            fn get_mut_element_dyn(&mut self, id: &str) -> Option<&mut dyn std::any::Any> {
                if self.id.as_deref() == Some(id) {
                    return Some(self.as_any_mut());
                }

                self.#content_field_name.get_mut_element_dyn(id)
            }

            fn id(&self) -> Option<&str> {
                self.id.as_deref()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    }
}

fn generate_without_content(
    name: &syn::Ident,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics graphics::widget::Queryable for #name #ty_generics #where_clause {
            fn get_element_dyn(&self, id: &str) -> Option<&dyn std::any::Any> {
                if self.id.as_deref() == Some(id) {
                    return Some(self.as_any());
                }
                None
            }

            fn get_mut_element_dyn(&mut self, id: &str) -> Option<&mut dyn std::any::Any> {
                if self.id.as_deref() == Some(id) {
                    return Some(self.as_any_mut());
                }
                None
            }

            fn id(&self) -> Option<&str> {
                self.id.as_deref()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    }
}
