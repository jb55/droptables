use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Lit, LitFloat, parse_macro_input, spanned::Spanned,
};

/// Variant attribute: #[probability(<expr>)]
#[proc_macro_derive(WeightedEnum, attributes(probability))]
pub fn derive_weighted_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(
            input.ident.span(),
            "WeightedEnum can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    // Collect (variant_ident, weight_expr)
    let mut entries = Vec::new();

    for variant in &data_enum.variants {
        // Only fieldless enums are supported (drop tables are usually C-like)
        match &variant.fields {
            Fields::Unit => {}
            _ => {
                return syn::Error::new(
                    variant.span(),
                    "WeightedEnum only supports fieldless variants",
                )
                .to_compile_error()
                .into();
            }
        }

        // Find #[probability(...)]
        let mut weight_expr: Option<Expr> = None;
        for Attribute { meta, .. } in &variant.attrs {
            if meta.path().is_ident("probability") {
                match meta {
                    syn::Meta::List(list) => {
                        // Parse inside as an expression (e.g., 1.0/100.0 or 1/100)
                        let expr = syn::parse2::<Expr>(list.tokens.clone()).map_err(|e| {
                            syn::Error::new(list.span(), format!("invalid probability expr: {e}"))
                        });
                        match expr {
                            Ok(e) => weight_expr = Some(e),
                            Err(err) => return err.to_compile_error().into(),
                        }
                    }
                    _ => {
                        return syn::Error::new(meta.span(), "use #[probability(<expr>)]")
                            .to_compile_error()
                            .into();
                    }
                }
            }
        }
        if weight_expr.is_none() {
            return syn::Error::new(variant.span(), "missing #[probability(...)] on variant")
                .to_compile_error()
                .into();
        }

        let ident = &variant.ident;
        let expr = weight_expr.unwrap();

        // Upgrade integer literals to floats so 1/100 => 1.0/100.0
        let expr_f64 = to_f64_expr(expr);

        entries.push(quote! { (Self::#ident, (#expr_f64)) });
    }

    // Generate const ENTRIES and helper droptable() inherent as sugar.
    let expanded = quote! {
        impl droptables::WeightedEnum for #enum_ident {
            const ENTRIES: &'static [(Self, f64)] = &[
                #(#entries),*
            ];
        }

        impl #enum_ident {
            /// Build a `DropTable<#enum_ident>` from annotated probabilities.
            pub fn droptable() -> ::core::result::Result<droptables::DropTable<Self>, droptables::ProbError>
            where
                Self: Copy
            {
                <Self as droptables::WeightedEnum>::droptable()
            }
        }
    };

    expanded.into()
}

/// Recursively rewrite integer literals to floating-point (e.g., 1 -> 1.0),
/// so that expressions like `1/100` use FP division.
fn to_f64_expr(mut e: Expr) -> Expr {
    match e {
        Expr::Lit(ref mut el) => {
            if let Lit::Int(int) = &el.lit {
                // 1 -> 1.0 (preserve span)
                let s = format!("{}{}", int.base10_digits(), ".0");
                el.lit = Lit::Float(LitFloat::new(&s, int.span()));
            }
            e
        }
        Expr::Binary(mut b) => {
            b.left = Box::new(to_f64_expr(*b.left));
            b.right = Box::new(to_f64_expr(*b.right));
            Expr::Binary(b)
        }
        Expr::Paren(mut p) => {
            p.expr = Box::new(to_f64_expr(*p.expr));
            Expr::Paren(p)
        }
        Expr::Unary(mut u) => {
            u.expr = Box::new(to_f64_expr(*u.expr));
            Expr::Unary(u)
        }
        Expr::Group(mut g) => {
            g.expr = Box::new(to_f64_expr(*g.expr));
            Expr::Group(g)
        }
        _ => e,
    }
}
