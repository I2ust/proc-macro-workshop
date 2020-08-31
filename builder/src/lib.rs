use proc_macro::TokenStream;
use quote::quote;
use syn::{
	parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed,
	GenericArgument, Ident, Lit, Meta, NestedMeta, PathArguments, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
	fn extract_ident_from_type(t: &Type) -> Option<&Ident> {
		if let Type::Path(p) = t {
			if let Some(s) = p.path.segments.first() {
				return Some(&s.ident);
			}
		}

		None
	}
	fn inner_type(t: &Type) -> Option<&Type> {
		if let Type::Path(p) = t {
			if let Some(s) = p.path.segments.first() {
				if let PathArguments::AngleBracketed(a) = &s.arguments {
					if let Some(GenericArgument::Type(t)) = a.args.first() {
						return Some(t);
					}
				}
			}
		}

		None
	}

	let ast = parse_macro_input!(input as DeriveInput);

	// eprint!("{:#?}", ast);

	let name = &ast.ident;
	let fields = if let Data::Struct(DataStruct {
		fields: Fields::Named(FieldsNamed { ref named, .. }),
		..
	}) = ast.data
	{
		named
	} else {
		unreachable!();
	};
	let builder_name = Ident::new(&format!("{}Builder", name), name.span());
	let builder_fields = fields.iter().map(|f| {
		let field_name = &f.ident;
		let field_type = &f.ty;

		if let Some(t) = extract_ident_from_type(&f.ty) {
			let t = t.to_string();

			if &t == "Option" || &t == "Vec" {
				return quote! { #field_name: #field_type };
			}
		}

		quote! { #field_name: ::core::option::Option<#field_type> }
	});
	let builder_build_fields = fields.iter().map(|f| {
		let field_name = &f.ident;

		if let Some(t) = extract_ident_from_type(&f.ty) {
			let t = t.to_string();

			if &t == "Option" || &t == "Vec" {
				return quote! {
					#field_name: self.#field_name.clone()
				};
			}
		}

		quote! {
			#field_name: self
				.#field_name
				.clone()
				.ok_or(concat!("`", stringify!(#field_name), "` is NOT set!"))?
		}
	});
	let mut builder_attr_method_names = vec![];
	let builder_attr_methods = fields
		.iter()
		.filter_map(|f| {
			if !f.attrs.is_empty() {
				let mut i = 0;

				if let Some(a) = f.attrs.iter().find(|a| {
					if let Some(p) = a
						.path
						.segments
						.iter()
						.position(|p| &p.ident.to_string() == "builder")
					{
						i = p;

						true
					} else {
						false
					}
				}) {
					if let Ok(Meta::List(l)) = a.parse_meta() {
						if let Some(n) = l.nested.first() {
							if let NestedMeta::Meta(Meta::NameValue(n)) = n {
								if let Some(i) = n.path.get_ident() {
									if &i.to_string() == "each" {
										if let Lit::Str(s) = &n.lit {
											builder_attr_method_names.push(s.value());

											let field_name = &f.ident;
											let each_name = Ident::new(&s.value(), s.span());
											let inner_type =
												extract_ident_from_type(inner_type(&f.ty).unwrap())
													.unwrap();

											return Some(quote! {
												pub fn #each_name(&mut self, #each_name: #inner_type) -> &mut Self {
													self.#field_name.push(#each_name);

													self
												}
											});
										} else {
											return None;
										}
									} else {
										return Some(
											Error::new(
												l.span(),
												"expected `builder(each = \"...\")`",
											)
											.to_compile_error(),
										);
									}
								}
							}
						}
					} else {
						return None;
					}
				}
			}

			None
		})
		.collect::<Vec<_>>();
	let builder_methods = fields
		.iter()
		.filter_map(|f| {
			let field_name = &f.ident;

			if let Some(field_name) = field_name {
				if builder_attr_method_names.contains(&field_name.to_string()) {
					return None;
				}
			}

			let field_type = &f.ty;

			if let Some(t) = extract_ident_from_type(&f.ty) {
				let t = t.to_string();

				if &t == "Vec" {
					return Some(quote! {
						pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
							self.#field_name = #field_name;

							self
						}
					});
				} else if &t == "Option" {
					let inner_type = extract_ident_from_type(inner_type(&f.ty).unwrap()).unwrap();

					return Some(quote! {
						pub fn #field_name(&mut self, #field_name: #inner_type) -> &mut Self {
							self.#field_name = ::core::option::Option::Some(#field_name);

							self
						}
					});
				}
			}

			Some(quote! {
				pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
					self.#field_name = ::core::option::Option::Some(#field_name);

					self
				}
			})
		})
		.collect::<Vec<_>>();

	(quote! {
		impl #name {
			pub fn builder() -> #builder_name {
				#builder_name::default()
			}
		}

		#[derive(Default)]
		pub struct #builder_name {
			#(#builder_fields),*
		}
		impl #builder_name {
			#(#builder_methods)*
			#(#builder_attr_methods)*

			pub fn build(&mut self) -> ::core::result::Result<#name, &'static str> {
				::core::result::Result::Ok(#name { #(#builder_build_fields),* })
			}
		}
	})
	.into()
}
