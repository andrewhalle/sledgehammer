use proc_macro::TokenStream;
use std::mem;

use quote::quote;
use syn::{parse_macro_input, Block, Expr, ExprForLoop, Item, ItemFn, Signature, Stmt};

fn pre_expr(expr: &Expr) -> String {
    match expr {
        Expr::ForLoop(for_loop) => {
            let ExprForLoop { pat, expr, .. } = &for_loop;
            let mut s = quote!( for #pat in #expr ).to_string();
            s.push_str(" {");

            s
        }
        _ => quote!(#expr).to_string(),
    }
}

fn post_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::ForLoop(_) => Some(String::from("}")),
        _ => None,
    }
}

fn transform_block(fn_name: &str, mut block: Block) -> Block {
    let statements = mem::take(&mut block.stmts);
    for statement in statements {
        block.stmts.push(pre(fn_name, &statement));
        let post_stmt = post(fn_name, &statement);
        block.stmts.push(transform(fn_name, statement));
        if let Some(stmt) = post_stmt {
            block.stmts.push(stmt);
        }
    }

    block
}

fn transform_expr(fn_name: &str, expr: Expr) -> Expr {
    match expr {
        Expr::ForLoop(for_loop) => {
            let ExprForLoop {
                attrs,
                label,
                for_token,
                pat,
                in_token,
                expr,
                body,
            } = for_loop;
            let body = transform_block(fn_name, body);
            Expr::ForLoop(ExprForLoop {
                attrs,
                label,
                for_token,
                pat,
                in_token,
                expr,
                body,
            })
        }
        _ => expr,
    }
}

fn pre(fn_name: &str, stmt: &Stmt) -> Stmt {
    let pre_str = match stmt {
        Stmt::Expr(expr) => pre_expr(expr),
        _ => quote!(#stmt).to_string(),
    };

    let dbg_info = format!("[SLEDGEHAMMER {}] {}", fn_name, pre_str);
    let code = quote! {
        eprintln!("{}", #dbg_info);
    };

    syn::parse_str(&code.to_string()).expect("Not a statement")
}

fn post(fn_name: &str, stmt: &Stmt) -> Option<Stmt> {
    let post_str = match stmt {
        Stmt::Expr(expr) => post_expr(expr),
        _ => None,
    };

    let code = post_str.map(|s| {
        let dbg_info = format!("[SLEDGEHAMMER {}] {}", fn_name, s);
        quote! {
            eprintln!("{}", #dbg_info);
        }
    });

    code.map(|code| syn::parse_str(&code.to_string()).expect("Not a statement"))
}

fn transform(fn_name: &str, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(expr) => Stmt::Expr(transform_expr(fn_name, expr)),
        _ => stmt,
    }
}

#[proc_macro_attribute]
pub fn sledgehammer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);

    let output = match input {
        Item::Fn(item_fn) => {
            let ItemFn {
                attrs,
                vis,
                sig,
                block,
            } = item_fn;
            let Signature { ident: fn_name, .. } = &sig;
            let fn_name = fn_name.to_string();

            let block = transform_block(&fn_name, *block);

            quote! {
                #(#attrs)* #vis #sig #block
            }
        }
        _ => panic!(),
    };

    output.into()
}
