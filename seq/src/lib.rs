#![allow(unused)]

use std::ops::Range;

use proc_macro::TokenStream;
use proc_macro2::{Group, Literal, TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;
use syn::{
	braced,
	parse::{Parse, ParseStream},
	parse_macro_input, ExprBlock, Ident, LitInt, Token,
};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as Seq);
	let output: TokenStream2 = input.into();

	output.into()
}

struct Seq {
	ident: Ident,
	range: Range<u16>,
	token_stream: TokenStream2,
}
impl Seq {
	fn repeat(&self) -> TokenStream2 {
		let mut token_stream = TokenStream2::new();

		for n in self.range.clone() {
			token_stream.extend(Self::generate(self.token_stream.clone(), &self.ident, n));
		}

		token_stream
	}

	fn generate(token_stream: TokenStream2, ident: &Ident, n: u16) -> TokenStream2 {
		token_stream
			.clone()
			.into_iter()
			.map(|tt| match &tt {
				TokenTree::Ident(i) if i == ident => Literal::u16_unsuffixed(n).into(),
				TokenTree::Group(g) => {
					Group::new(g.delimiter(), Self::generate(g.stream(), ident, n)).into()
				}
				_ => tt,
			})
			.collect()
	}
}
impl Parse for Seq {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ident = Ident::parse(input)?;

		<Token![in]>::parse(input)?;

		let range = {
			let s = LitInt::parse(input)?.base10_parse::<u16>()?;
			let e = if input.peek(Token![..=]) {
				<Token![..=]>::parse(input)?;

				1
			} else {
				<Token![..]>::parse(input)?;

				0
			} + LitInt::parse(input)?.base10_parse::<u16>()?;

			s..e
		};
		let token_stream = {
			let c;
			braced!(c in input);

			TokenStream2::parse(&c)?
		};

		Ok(Seq {
			ident,
			range,
			token_stream,
		})
	}
}
impl Into<TokenStream2> for Seq {
	fn into(self) -> TokenStream2 {
		self.repeat()
	}
}
