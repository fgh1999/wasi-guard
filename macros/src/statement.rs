use alloc::collections::BTreeMap;

use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Result,
};

#[derive(Clone)]
pub enum Action {
    Allow,
    Log,
    // With a user-defined errno.
    ReturnErrno(syn::Expr),
    Kill,
}
impl Action {
    pub const fn is_kill(&self) -> bool {
        matches!(self, Action::Kill)
    }
    pub const fn is_allow(&self) -> bool {
        matches!(self, Action::Allow)
    }
}

impl Parse for Action {
    fn parse(input: ParseStream) -> Result<Self> {
        let action = input.parse::<Ident>()?;
        match action.to_string().to_lowercase().as_str() {
            "allow" => Ok(Action::Allow),
            "log" => Ok(Action::Log),
            "kill" => Ok(Action::Kill),
            "ret_err" | "ret_errno" | "return_errno" => {
                let errno;
                syn::parenthesized!(errno in input);
                Ok(Action::ReturnErrno(errno.parse()?))
            }
            _ => Err(input.error("expected `allow`, `log`, `kill`, or `return_errno`")),
        }
    }
}
impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Action::Allow => quote! { wasi_guard::policy::action::Action::Allow },
            Action::Log => quote! { wasi_guard::policy::action::Action::Log },
            Action::ReturnErrno(errno) => {
                quote! { wasi_guard::policy::action::Action::ReturnErrno((#errno) as wasi_guard::policy::action::WasiErrno) }
            }
            Action::Kill => quote! { wasi_guard::policy::action::Action::Kill },
        }
        .to_tokens(tokens)
    }
}

/// Bound expression for a WASI guard statement,
/// used to check the arguments of a WASI function.
pub enum Bound {
    /// A closure expression that returns bool: `|a: i32, b: i32| a + b < 256`.
    Closure(syn::ExprClosure),
    /// A function call expression: `invoke(a, b)`.
    Call(syn::ExprCall),
    /// A path like `bounds::must_be_stdio` which may check a file descriptor.
    ///
    /// A plain identifier like `x` is a path of length 1.
    Path(syn::ExprPath),
    /// An `if` expression with an optional `else` block: `if expr { ... }
    /// else { ... }`.
    If(syn::ExprIf),
    /// A square bracketed indexing expression: `bounds[2]`.
    Index(syn::ExprIndex),
    /// Access of a named struct field (`obj.k`) or unnamed tuple struct
    /// field (`obj.0`).
    Field(syn::ExprField),
}
impl Parse for Bound {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        match expr {
            syn::Expr::Closure(closure) => Ok(Bound::Closure(closure)),
            syn::Expr::Path(path) => Ok(Bound::Path(path)),
            syn::Expr::If(if_expr) => Ok(Bound::If(if_expr)),
            syn::Expr::Call(call) => Ok(Bound::Call(call)),
            syn::Expr::Index(index) => Ok(Bound::Index(index)),
            syn::Expr::Field(field) => Ok(Bound::Field(field)),
            _ => Err(input.error("expected a closure, call, path, if, index, or field expression")),
        }
    }
}
impl ToTokens for Bound {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Bound::Closure(closure) => closure.to_tokens(tokens),
            Bound::Path(path) => path.to_tokens(tokens),
            Bound::If(if_expr) => if_expr.to_tokens(tokens),
            Bound::Call(call) => call.to_tokens(tokens),
            Bound::Index(index) => index.to_tokens(tokens),
            Bound::Field(field) => field.to_tokens(tokens),
        }
    }
}

pub struct WasiStatement {
    pub action: Action,
    pub wasi: syn::Ident,
    pub bounds: Vec<Bound>,
    pub arg_types: Vec<syn::PatType>,
}
impl WasiStatement {
    pub fn new(wasi: syn::Ident, action: Action) -> Self {
        Self {
            action,
            wasi,
            bounds: Vec::new(),
            arg_types: Vec::new(),
        }
    }

    pub fn must_be_killed(&self) -> bool {
        self.action.is_kill() && self.bounds.is_empty()
    }
}

impl Parse for WasiStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        let action = input.parse()?;
        let wasi = input.parse()?;
        let bounds: Vec<Bound> = if input.peek(syn::Token![where]) {
            input.parse::<syn::Token![where]>().unwrap();
            if input.peek(syn::Token![;]) {
                Vec::new()
            } else {
                Punctuated::<Bound, syn::Token![,]>::parse_separated_nonempty(input)?
                    .into_iter()
                    .collect()
            }
        } else {
            Vec::new()
        };

        // If there is at least one bound that is a closure with typed arguments,
        // we can parse the type of the arguments.
        let mut arg_types: Vec<syn::PatType> = Vec::new();
        for closure in bounds.iter().filter_map(|b| match b {
            Bound::Closure(c) => Some(c),
            _ => None,
        }) {
            arg_types.clear();
            for arg in closure.inputs.iter() {
                match arg {
                    syn::Pat::Type(pat) => arg_types.push(pat.clone()),
                    _ => {
                        arg_types.clear();
                        break;
                    }
                }
            }
            if !arg_types.is_empty() {
                break;
            }
        }
        if arg_types.is_empty() && !bounds.is_empty() {
            return Err(input.error("expected a closure with all arguments typed"));
        }

        let mut stat = WasiStatement::new(wasi, action);
        stat.bounds = bounds;
        stat.arg_types = arg_types;
        Ok(stat)
    }
}

impl ToTokens for WasiStatement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            wasi,
            bounds,
            action,
            ..
        } = self;
        let bounds = if bounds.is_empty() {
            proc_macro2::TokenStream::new()
        } else {
            quote! { where #(#bounds),* }
        };

        quote! { wasi_guard::statement!(#wasi #bounds => #action) }.to_tokens(tokens);
    }
}

pub struct Policy {
    pub default_action: Action,
    /// { wasi_ident -> statements }
    pub statements: BTreeMap<syn::Ident, Vec<WasiStatement>>,

    wasi_names: Vec<String>,
}
impl Policy {
    fn has_wasi_named(&self, wasi_name: &str) -> bool {
        self.wasi_names.iter().any(|ident| ident == wasi_name)
    }
}

impl Parse for Policy {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Token![default]>()?;
        input.parse::<syn::Token![=]>()?;
        let default_action = input.parse()?;

        let mut statements: Vec<WasiStatement> = Vec::new();
        while !input.is_empty() {
            // ignore heading semicolons
            while input.peek(syn::Token![;]) {
                input.parse::<syn::Token![;]>().unwrap();
            }
            if !input.is_empty() {
                statements.push(input.parse()?);
            }
        }
        let statements: BTreeMap<syn::Ident, Vec<WasiStatement>> =
            statements
                .into_iter()
                .fold(BTreeMap::new(), |mut map, stmt| {
                    map.entry(stmt.wasi.clone()).or_default().push(stmt);
                    map
                });
        let wasi_names = statements.keys().map(|ident| ident.to_string()).collect();

        Ok(Self {
            default_action,
            statements,
            wasi_names,
        })
    }
}

// Only used when the type can not be inferred from the arguments in bounds
fn get_path_of_default_param_type(wasi_name: &str) -> proc_macro2::TokenStream {
    let wasi_guard_lib = format_ident!("wasi_guard");
    let wasi_lib = format_ident!("wasi");
    let type_name = format_ident!("{}_params_default_t", wasi_name.to_lowercase());
    quote! { #wasi_guard_lib::#wasi_lib::#type_name }
}

impl ToTokens for Policy {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let default_action = self.default_action.clone();
        let specified_guards = self.statements.iter().map(|(wasi, stmts)| {
            let wasi_name = wasi.to_string();
            let guard_name = format_ident!("WASI_GUARD_{}", wasi_name.to_uppercase());
            let param_type = if stmts[0].arg_types.is_empty() {
                let param_type_path = get_path_of_default_param_type(wasi_name.as_str());
                quote! { #param_type_path }
            } else {
                let param_types: Vec<&Box<syn::Type>> = stmts[0].arg_types.iter().map(|pat| &pat.ty).collect();
                quote! { (#(#param_types,)*) }
            };
            let param_type_name = format_ident!("{}_guard_params_t", wasi_name.to_lowercase());
            quote! {
                pub type #param_type_name = #param_type;
                wasi_guard::policy::lazy_static! {
                    pub static ref #guard_name: Option<wasi_guard::policy::WasiGuard<'static, #param_type_name>> =
                        Some(wasi_guard::policy::WasiGuard::from_arr([
                            #(#stmts),*
                        ]));
                }
            }
        });

        let rest_wasis: Vec<String> = wasi::WASI_NAMES
            .iter()
            .filter_map(|wasi_name| {
                if !self.has_wasi_named(wasi_name) {
                    Some(wasi_name.to_string())
                } else {
                    None
                }
            })
            .collect();
        let default_guards = rest_wasis.iter().map(|wasi_name| {
            let guard_name = format_ident!("WASI_GUARD_{}", wasi_name.to_uppercase());
            let param_type = get_path_of_default_param_type(wasi_name);
            let param_type_name = format_ident!("{}_guard_params_t", wasi_name.to_lowercase());

            quote! {
                pub type #param_type_name = #param_type;
                pub const #guard_name: Option<wasi_guard::policy::WasiGuard<'static, #param_type_name>> = None;
            }
        });

        let must_be_killed = {
            let mut must_be_killed: Vec<_> = self
                .statements
                .iter()
                .filter(|(_, stmts)| stmts.iter().any(|stmt| stmt.must_be_killed()))
                .map(|(wasi, _)| wasi.to_string())
                .collect();

            if default_action.is_kill() {
                must_be_killed.extend(rest_wasis.clone());
            }

            let vec_len = must_be_killed.len();
            quote! {
                pub const MUST_BE_KILLED_WASIS: [&str; #vec_len] = [ #(#must_be_killed),*];
            }
        };

        quote! {
            pub const DEFUALT_ACTION: wasi_guard::policy::action::Action = #default_action;
            #(#specified_guards)*
            #(#default_guards)*
            #must_be_killed
        }
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wasmedge_sock_abis() {
        #[cfg(feature = "wasmedge-sock")]
        assert!(wasi::WASI_NAMES.iter().any(|name| name == &"sock_listen"));
        #[cfg(not(feature = "wasmedge-sock"))]
        assert!(!wasi::WASI_NAMES.iter().any(|name| name == &"sock_listen"));
    }
}
