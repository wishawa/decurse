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

struct Parsed(ItemFn);

impl Parse for Parsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let f: ItemFn = input.parse()?;
        if let Some(a) = &f.sig.asyncness {
            return Err(Error::new(a.span, "Async function not supported."));
        }
        Ok(Self(f))
    }
}

struct Folder {
    ident: Ident,
}

impl Folder {
    fn new(ident: &Ident) -> Self {
        Self {
            ident: ident.clone(),
        }
    }
}

fn generate_call(call: &ExprCall) -> Expr {
    parse_quote!(::decurse::recurse!(#call))
}

impl Fold for Folder {
    fn fold_expr(&mut self, node: Expr) -> Expr {
        if let Expr::Call(c) = &node {
            if let Expr::Path(p) = &*c.func {
                if p.path.segments.last().unwrap().ident == self.ident {
                    return generate_call(c);
                }
            }
        }
        fold_expr(self, node)
    }
}

impl Parsed {
    fn generate(self) -> Result<TokenStream, Error> {
        let mut new = self.0;
        let name = new.sig.ident.clone();
        let sig = new.sig.clone();
        let args = new.sig.inputs.clone();
        let arg_pats: Punctuated<Pat, Comma> = args
            .iter()
            .filter_map(|a| match a {
                FnArg::Typed(t) => Some(*t.pat.clone()),
                _ => None,
            })
            .collect();

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

        // Create wrapper
        Ok(quote! {
            #sig {
                #new
                ::decurse::execute( #name(#arg_pats) )
            }
        })
    }
}

#[proc_macro_attribute]
pub fn decurse(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let func = parse_macro_input!(item as Parsed);
    let generated = func.generate().unwrap();
    generated.into()
}
