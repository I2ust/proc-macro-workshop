// --- proc_macro ---
use proc_macro::TokenStream;
// --- crates ---
use proc_macro2::Span;
use syn::{
	parse::Error,
	spanned::Spanned,
	visit_mut::{self, VisitMut},
	ExprMatch, Item, ItemFn, Pat, Path,
};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, mut input: TokenStream) -> TokenStream {
	let token_stream = input.clone();
	let item = syn::parse_macro_input!(token_stream as Item);

	// dbg!(&item);

	if let Err(e) = impl_sorted(&item) {
		input.extend(TokenStream::from(e.into_compile_error()));
	}

	input
}

fn impl_sorted(item: &Item) -> Result<(), Error> {
	if let Item::Enum(e) = item {
		let mut idents = vec![];

		for v in &e.variants {
			let cur_i = v.ident.to_string();

			if let Some(i) = idents.iter().position(|i| i >= &cur_i) {
				return Err(Error::new(
					v.ident.span(),
					format!("{} should sort before {}", cur_i, idents[i]),
				));
			} else {
				idents.push(cur_i);
			}
		}
	} else {
		return Err(Error::new(
			Span::call_site(),
			"expected enum or match expression",
		));
	}

	Ok(())
}

struct SortedMatch(Result<(), Error>);
impl SortedMatch {
	fn new() -> Self {
		Self(Ok(()))
	}
}
impl VisitMut for SortedMatch {
	fn visit_expr_match_mut(&mut self, m: &mut ExprMatch) {
		m.attrs.retain(|a| {
			a.path
				.get_ident()
				.map(|i| &i.to_string() != "sorted")
				.unwrap_or_default()
		});

		// dbg!(&m);

		let mut pats = <Vec<(Span, String)>>::new();

		for (i, a) in m.arms.iter().enumerate() {
			let path_to_string = |p: &Path| {
				p.segments
					.iter()
					.map(|s| s.ident.to_string())
					.collect::<Vec<_>>()
					.join("::")
			};
			let cur_p = match &a.pat {
				Pat::Ident(i) => (i.span(), i.ident.to_string()),
				Pat::Slice(s) => {
					self.0 = Err(Error::new(s.span(), "unsupported by #[sorted]"));

					return;
				}
				Pat::TupleStruct(t) => (t.path.span(), path_to_string(&t.path)),
				Pat::Wild(w) => {
					if i != m.arms.len() - 1 {
						self.0 = Err(Error::new(w.span(), "_ should be the last pattern"));
					}

					return;
				}
				_ => unimplemented!(),
			};

			if let Some(i) = pats.iter().position(|p| &p.1 >= &cur_p.1) {
				self.0 = Err(Error::new(
					cur_p.0,
					format!("{} should sort before {}", cur_p.1, pats[i].1),
				));

				return;
			} else {
				pats.push(cur_p);
			}
		}

		visit_mut::visit_expr_match_mut(self, m);
	}
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
	let mut item_fn = syn::parse_macro_input!(input as ItemFn);

	// dbg!(&item_fn);

	let mut sorted_match = SortedMatch::new();

	sorted_match.visit_item_fn_mut(&mut item_fn);

	let mut token_stream = TokenStream::from(quote::quote! { #item_fn });

	if let Err(e) = sorted_match.0 {
		token_stream.extend(TokenStream::from(e.into_compile_error()));
	}

	token_stream
}
