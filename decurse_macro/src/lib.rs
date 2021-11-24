use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    fold::{fold_expr, Fold},
    parse::Parse,
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Error, Expr, ExprCall, FnArg, ItemFn, Pat, Stmt, Token, Visibility,
};
struct Parsed(TokenStream);

impl Parse for Parsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let f: ItemFn = input.parse()?;
        if let Some(a) = &f.sig.asyncness {
            return Err(Error::new(a.span, "Decurse: async function not supported."));
        }
        let generated = generate(f)?;
        Ok(Self(generated))
    }
}

struct Folder {
    ident: Ident,
    closure_nested: usize,
    async_nested: usize,
    errors: Vec<Error>,
}

impl Folder {
    fn new(ident: &Ident) -> Self {
        Self {
            ident: ident.clone(),
            closure_nested: 0,
            async_nested: 0,
            errors: Vec::new(),
        }
    }
}

fn generate_call(call: &ExprCall) -> Expr {
    parse_quote!(::decurse::recurse!(#call))
}

impl Fold for Folder {
    fn fold_expr(&mut self, node: Expr) -> Expr {
        match &node {
            Expr::Call(c) => {
                if let Expr::Path(p) = &*c.func {
                    let ident = &p.path.segments.last().unwrap().ident;
                    if ident == &self.ident {
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
                        return generate_call(c);
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
}

fn generate(mut new: ItemFn) -> Result<TokenStream, Error> {
    // Extracting infos
    let name = new.sig.ident.clone();
    let sig = new.sig.clone();
    let args = new.sig.inputs.clone();
    let arg_names: Punctuated<Pat, Comma> = args
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(t) => Some(*t.pat.clone()),
            _ => None,
        })
        .collect();

    // Modifying signature
    new.vis = Visibility::Inherited;
    new.sig.asyncness = Some(Token!(async)(Span::call_site()));

    // Modifying body
    let mut folder = Folder::new(&name);
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
    Ok(quote! {
        #sig {
            #new
            ::decurse::execute(#name(#arg_names))
        }
    })
}

#[proc_macro_attribute]
pub fn decurse(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as Parsed);
    parsed.0.into()
}
