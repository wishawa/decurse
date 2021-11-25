//! This crate provide macros for the [`decurse` crate](https://crates.io/crates/decurse).
//! Please see there for more details.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
	fold::{fold_expr, fold_fn_arg, fold_item_fn, Fold},
	parse::Parse,
	parse_macro_input, parse_quote,
	punctuated::Punctuated,
	token::Comma,
	Error, Expr, FnArg, Generics, ItemFn, Pat, PatIdent, Signature, Stmt, Token, Visibility,
};
struct Parsed(ItemFn);

impl Parse for Parsed {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut f: ItemFn = input.parse()?;
		if let Some(a) = &f.sig.asyncness {
			return Err(Error::new(a.span, "Decurse: async function not supported."));
		}
		let mut arg_checker = ArgChecker::new();
		f.sig = arg_checker.fold_signature(f.sig);
		if let Some(err) = arg_checker.errors.into_iter().next() {
			return Err(err);
		}
		Ok(Self(f))
	}
}

fn remove_lifetimes(sig: &Signature) -> Generics {
	let mut generics = sig.generics.clone();
	generics.params = generics
		.params
		.into_iter()
		.filter_map(|p| match p {
			syn::GenericParam::Type(t) => Some(syn::GenericParam::Type(t)),
			syn::GenericParam::Lifetime(_) => None,
			syn::GenericParam::Const(v) => Some(syn::GenericParam::Const(v)),
		})
		.collect();
	generics
}

struct ArgChecker {
	errors: Vec<Error>,
}
impl ArgChecker {
	fn new() -> Self {
		Self { errors: Vec::new() }
	}
}

impl Fold for ArgChecker {
	fn fold_fn_arg(&mut self, i: FnArg) -> FnArg {
		match &i {
			FnArg::Receiver(s) => self.errors.push(Error::new(
				s.self_token.span,
				"Decurse: method not supported.",
			)),
			FnArg::Typed(ty) => match &*ty.ty {
				syn::Type::ImplTrait(impl_trait) => {
					self.errors.push(Error::new(
						impl_trait.impl_token.span,
						"Decurse: impl Trait argument not supported.",
					));
				}
				syn::Type::Macro(mac) => {
					self.errors.push(Error::new(
						mac.mac.bang_token.span,
						"Decurse: macro argument type not supported.",
					));
				}
				_ => {}
			},
		}
		fold_fn_arg(self, i)
	}
}

struct Folder {
	use_unsound_impl: bool,
	sig: Signature,
	closure_nested: usize,
	async_nested: usize,
	fn_nested: usize,
	errors: Vec<Error>,
}

impl Folder {
	fn new(sig: Signature, use_unsound_impl: bool) -> Self {
		Self {
			use_unsound_impl,
			sig,
			closure_nested: 0,
			async_nested: 0,
			fn_nested: 0,
			errors: Vec::new(),
		}
	}
	fn generate_call(&self, args: &Punctuated<Expr, Comma>) -> Expr {
		let func = &self.sig.ident;
		let generics_wo_lt = remove_lifetimes(&self.sig);
		let spi = generics_wo_lt.split_for_impl();
		let tbfs = &spi.1.as_turbofish();
		if self.use_unsound_impl {
			parse_quote!(::decurse::for_macro_only_recurse_unsound!(#func#tbfs, (#args)))
		} else {
			parse_quote!(::decurse::for_macro_only_recurse_sound!(#func#tbfs, (#args)))
		}
	}
}

impl Fold for Folder {
	fn fold_expr(&mut self, node: Expr) -> Expr {
		match &node {
			Expr::Call(c) => {
				if let Expr::Path(p) = &*c.func {
					let ident = &p.path.segments.first().unwrap().ident;
					let l = p.path.segments.len();
					if l == 1 && ident == &self.sig.ident {
						if self.closure_nested > 0 {
							self.errors.push(Error::new(
								ident.span(),
								"Decurse: recursive call inside closure not supported.",
							));
						}
						if self.async_nested > 0 {
							self.errors.push(Error::new(
								ident.span(),
								"Decurse: recursive call inside async block not supported.",
							));
						}
						if self.fn_nested > 0 {
							self.errors.push(Error::new(
								ident.span(),
								"Decurse: recursive call in sub-function not supported.",
							))
						}
						return self.generate_call(&c.args);
					}
				}
				fold_expr(self, node)
			}
			Expr::Closure(_) => {
				self.closure_nested += 1;
				let r = fold_expr(self, node);
				self.closure_nested -= 1;
				r
			}
			Expr::Async(_) => {
				self.async_nested += 1;
				let r = fold_expr(self, node);
				self.async_nested -= 1;
				r
			}
			_ => fold_expr(self, node),
		}
	}
	fn fold_item_fn(&mut self, i: ItemFn) -> ItemFn {
		self.fn_nested += 1;
		let r = fold_item_fn(self, i);
		self.fn_nested -= 1;
		r
	}
}

fn generate(mut new: ItemFn, use_unsound_impl: bool) -> Result<TokenStream, Error> {
	// Extracting infos
	let name = new.sig.ident.clone();
	let generics_wo_lt = remove_lifetimes(&new.sig);
	let spi = generics_wo_lt.split_for_impl();
	let tbfs = &spi.1.as_turbofish();
	let mut wrapping_sig = new.sig.clone();
	wrapping_sig
		.inputs
		.iter_mut()
		.enumerate()
		.for_each(|(i, a)| match a {
			FnArg::Typed(t) => {
				let ident = Ident::new(&format!("arg_{}", i), Span::call_site());
				let id = PatIdent {
					attrs: Vec::new(),
					by_ref: None,
					mutability: None,
					ident,
					subpat: None,
				};
				t.pat = Box::new(Pat::Ident(id));
			}
			_ => {}
		});
	let arg_names =
		(0..new.sig.inputs.len()).map(|i| Ident::new(&format!("arg_{}", i), Span::call_site()));

	// Modifying signature
	new.vis = Visibility::Inherited;
	new.sig.asyncness = Some(Token!(async)(Span::call_site()));

	// Modifying body
	let mut folder = Folder::new(new.sig.clone(), use_unsound_impl);
	let stmts: Vec<Stmt> = new
		.block
		.stmts
		.into_iter()
		.map(|stmt| folder.fold_stmt(stmt))
		.collect();
	new.block.stmts = stmts;
	if let Some(e) = folder.errors.into_iter().next() {
		return Err(e);
	}

	// Create wrapper
	if use_unsound_impl {
		Ok(quote! {
			#wrapping_sig {
				#new
				::decurse::for_macro_only::unsound::execute(#name#tbfs(#(#arg_names),*))
			}
		})
	} else {
		Ok(quote! {
			#wrapping_sig {
				#new
				::decurse::for_macro_only::sound::execute(#name#tbfs(#(#arg_names),*))
			}
		})
	}
}

#[proc_macro_attribute]
pub fn decurse_sound(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let parsed = parse_macro_input!(item as Parsed);
	let generated = generate(parsed.0, false).unwrap_or_else(Error::into_compile_error);
	generated.into()
}

#[proc_macro_attribute]
pub fn decurse_unsound(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let parsed = parse_macro_input!(item as Parsed);
	let generated = generate(parsed.0, true).unwrap_or_else(Error::into_compile_error);
	generated.into()
}
