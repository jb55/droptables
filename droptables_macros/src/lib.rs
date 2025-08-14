use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Lit, Meta,
    MetaNameValue,
};

#[proc_macro_derive(WeightedEnum, attributes(odds, rest))]
pub fn derive_weighted_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(
            input.ident.span(),
            "WeightedEnum can only be derived for fieldless enums",
        )
        .to_compile_error()
        .into();
    };

    // Stage 1: parse attributes
    #[derive(Debug)]
    struct VarTmp {
        ident: syn::Ident,
        prob: Option<f64>,          // from #[odds="A/B"]
        is_rest: bool,              // from #[rest]
    }

    let mut tmp: Vec<VarTmp> = Vec::with_capacity(data_enum.variants.len());
    let mut rest_count = 0usize;

    for v in &data_enum.variants {
        match v.fields {
            Fields::Unit => {}
            _ => {
                return syn::Error::new(v.span(), "WeightedEnum only supports fieldless variants")
                    .to_compile_error()
                    .into();
            }
        }

        let mut prob: Option<f64> = None;
        let mut is_rest = false;

        for Attribute { meta, .. } in &v.attrs {
            if meta.path().is_ident("odds") {
                let Meta::NameValue(MetaNameValue { value, .. }) = meta else {
                    return syn::Error::new(
                        meta.span(),
                        r#"use #[odds = "A/B"] (string literal)"#,
                    )
                    .to_compile_error()
                    .into();
                };

                // In syn 2, value is an Expr; we need a string literal.
                let p = match &value {
                    syn::Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. }) => {
                        match parse_odds_str(&s.value()) {
                            Ok(p) => p,
                            Err(e) => return syn::Error::new(s.span(), e).to_compile_error().into(),
                        }
                    }
                    _ => {
                        return syn::Error::new(
                            value.span(),
                            r#"odds must be a string literal like "1/100""#,
                        )
                        .to_compile_error()
                        .into()
                    }
                };

                if p <= 0.0 || !p.is_finite() {
                    return syn::Error::new(
                        value.span(),
                        "odds must produce a positive, finite probability",
                    )
                    .to_compile_error()
                    .into();
                }
                if prob.replace(p).is_some() {
                    return syn::Error::new(meta.span(), "duplicate #[odds] on variant")
                        .to_compile_error()
                        .into();
                }
            } else if meta.path().is_ident("rest") {
                if is_rest {
                    return syn::Error::new(meta.span(), "duplicate #[rest] on variant")
                        .to_compile_error()
                        .into();
                }
                is_rest = true;
                rest_count += 1;
            }
        }

        if prob.is_none() && !is_rest {
            return syn::Error::new(
                v.span(),
                "each variant must have either #[odds=\"A/B\"] or #[rest]",
            )
            .to_compile_error()
            .into();
        }

        tmp.push(VarTmp {
            ident: v.ident.clone(),
            prob,
            is_rest,
        });
    }

    if rest_count > 1 {
        return syn::Error::new(
            enum_ident.span(),
            "at most one variant may use #[rest]",
        )
        .to_compile_error()
        .into();
    }

    // Stage 2: validate and materialize probabilities
    const EPS: f64 = 1e-6;
    let mut sum_known = 0.0f64;
    for v in &tmp {
        if let Some(p) = v.prob {
            sum_known += p;
        }
    }

    let finalized: Vec<(syn::Ident, f32)> = if rest_count == 1 {
        if sum_known > 1.0 + EPS {
            return syn::Error::new(
                enum_ident.span(),
                format!(
                    "sum of specified odds exceeds 1: {:.8}. Remove a variant or adjust odds.",
                    sum_known
                ),
            )
            .to_compile_error()
            .into();
        }
        let rest_val = 1.0 - sum_known;
        if rest_val < -EPS {
            return syn::Error::new(enum_ident.span(), "computed #[rest] is negative")
                .to_compile_error()
                .into();
        }
        tmp.into_iter()
            .map(|v| {
                let p = if v.is_rest {
                    if rest_val < 0.0 && rest_val.abs() <= EPS { 0.0 } else { rest_val }
                } else {
                    v.prob.unwrap()
                };
                (v.ident, p as f32)
            })
            .collect()
    } else {
        // No #[rest]: require exact sum ~ 1
        if (sum_known - 1.0).abs() > EPS {
            return syn::Error::new(
                enum_ident.span(),
                format!("probabilities must sum to 1.0 (±{EPS}): got {:.8}", sum_known),
            )
            .to_compile_error()
            .into();
        }
        tmp.into_iter()
            .map(|v| (v.ident, v.prob.unwrap() as f32))
            .collect()
    };

    // Stage 3: expand
    let entries = finalized.iter().map(|(ident, p)| {
        quote! { (Self::#ident, #p) }
    });

    let expanded = quote! {
        impl droptables::WeightedEnum for #enum_ident {
            const ENTRIES: &'static [(Self, f32)] = &[
                #(#entries),*
            ];
        }

        impl #enum_ident {
            /// Build a `DropTable<#enum_ident>` from literal per-variant probabilities.
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

// --- helpers ---

// Parse "A/B" (ints or floats), allow spaces around '/', A>0, B>0
fn parse_odds_str(s: &str) -> Result<f64, &'static str> {
    let s = s.trim();
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(r#"expected "A/B" (e.g., "1/100")"#);
    }
    let a = parse_num(parts[0].trim())?;
    let b = parse_num(parts[1].trim())?;
    if a <= 0.0 || b <= 0.0 {
        return Err("A and B must be positive");
    }
    Ok(a / b)
}

fn parse_num(s: &str) -> Result<f64, &'static str> {
    s.parse::<f64>().map_err(|_| "failed to parse number")
}
