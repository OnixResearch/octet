use std::collections::BTreeMap;
use std::path::Path;

use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::visit::Visit;

const MAX_TOKEN_QUEUE: usize = 4096;
const INITIAL_TOKEN_QUEUE_CAP: usize = 16;

pub(crate) struct SourceScanOutput {
    pub(crate) generated: BTreeMap<String, Vec<u8>>,
    pub(crate) unsupported: Vec<String>,
    pub(crate) incomplete: Vec<String>,
    pub(crate) module_locators: Vec<String>,
}

pub(crate) fn scan_rust_source(
    package_root: &Path,
    file_locator: &str,
    bytes: &[u8],
    settings: &super::policy::Settings,
) -> SourceScanOutput {
    let mut visitor = MacroInputVisitor {
        package_root,
        file_locator,
        settings,
        mod_dir_stack: Vec::with_capacity(INITIAL_TOKEN_QUEUE_CAP),
        out: SourceScanOutput {
            generated: BTreeMap::new(),
            unsupported: Vec::new(),
            incomplete: Vec::new(),
            module_locators: Vec::new(),
        },
    };
    match std::str::from_utf8(bytes)
        .map_err(|_| format!("source file `{file_locator}` is not valid UTF-8"))
        .and_then(|src| syn::parse_file(src).map_err(|_| format!("source file `{file_locator}` failed Rust syntax")))
    {
        Ok(file) => visitor.visit_file(&file),
        Err(err) => visitor.out.incomplete.push(err),
    }
    visitor.out
}

struct MacroInputVisitor<'a> {
    package_root: &'a Path,
    file_locator: &'a str,
    settings: &'a super::policy::Settings,
    mod_dir_stack: Vec<Vec<String>>,
    out: SourceScanOutput,
}

impl<'ast> Visit<'ast> for MacroInputVisitor<'_> {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node.path.segments.last().map(|seg| seg.ident.to_string()).unwrap_or_default();
        let tokens_str = node.tokens.to_string();
        self.dispatch_macro(&name, &tokens_str);
        self.scan_token_string(tokens_str);
        syn::visit::visit_macro(self, node);
    }

    fn visit_meta_list(&mut self, node: &'ast syn::MetaList) {
        self.scan_token_string(node.tokens.to_string());
        syn::visit::visit_meta_list(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_inline = node.content.is_some();
        let explicit = node.attrs.iter().filter_map(|attr| self.inspect_mod_meta(&attr.meta, is_inline)).last();
        if !is_inline {
            return syn::visit::visit_item_mod(self, node);
        }
        let next_dir = explicit.unwrap_or_else(|| {
            let mut dir = self.mod_dir_stack.last().cloned().unwrap_or_else(|| self.file_base());
            dir.push(node.ident.to_string());
            dir
        });
        self.mod_dir_stack.push(next_dir);
        syn::visit::visit_item_mod(self, node);
        self.mod_dir_stack.pop();
    }

    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        if ident == "OUT_DIR" {
            self.out.unsupported.push(format!("`{}` references `OUT_DIR`", self.file_locator));
        }
    }
}

impl MacroInputVisitor<'_> {
    fn dispatch_macro(&mut self, name: &str, tokens_str: &str) {
        match name {
            "include" | "include_str" | "include_bytes" => self.handle_include_macro(name, tokens_str),
            "env" | "option_env" => self.out.unsupported.push(format!("`{}` uses `{name}!`", self.file_locator)),
            _ => {}
        }
    }

    fn scan_token_string(&mut self, root_stream: String) {
        let mut queue = Vec::with_capacity(INITIAL_TOKEN_QUEUE_CAP);
        queue.push(root_stream);
        while let Some(stream_str) = queue.pop() {
            if (|input: syn::parse::ParseStream<'_>| {
                while !input.is_empty() && self.scan_next_token(input, &mut queue)? {}
                Ok(())
            })
            .parse_str(&stream_str)
            .is_err()
            {}
        }
    }

    fn scan_next_token(&mut self, input: syn::parse::ParseStream<'_>, queue: &mut Vec<String>) -> syn::Result<bool> {
        if input.peek(syn::Ident::peek_any) {
            let ident = input.call(syn::Ident::parse_any)?;
            if ident == "OUT_DIR" {
                self.out.unsupported.push(format!("`{}` references `OUT_DIR`", self.file_locator));
            }
            if input.peek(syn::Token![!]) {
                let _bang: syn::Token![!] = input.parse()?;
                if let Some(inner_str) = parse_delimited_string(input)? {
                    self.dispatch_macro(&ident.to_string(), &inner_str);
                    return Ok(push_stream(queue, inner_str, &mut self.out.incomplete, self.file_locator));
                }
            }
            return Ok(true);
        }
        if input.peek(syn::Token![#]) {
            let _pound: syn::Token![#] = input.parse()?;
            if input.peek(syn::token::Bracket)
                && let Some(attr_str) = parse_delimited_string(input)?
            {
                if let Some(is_inline) = tokens_follow_mod_kind(&input.fork())?
                    && let Ok(meta) = syn::parse_str::<syn::Meta>(&attr_str)
                {
                    let _ = self.inspect_mod_meta(&meta, is_inline);
                }
                return Ok(push_stream(queue, attr_str, &mut self.out.incomplete, self.file_locator));
            }
            return Ok(true);
        }
        if let Some(inner_str) = parse_delimited_string(input)? {
            return Ok(push_stream(queue, inner_str, &mut self.out.incomplete, self.file_locator));
        }
        input.step(|cursor| Ok(((), cursor.token_tree().map_or(*cursor, |(_, next)| next))))?;
        Ok(true)
    }

    fn inspect_mod_meta(&mut self, meta: &syn::Meta, is_inline: bool) -> Option<Vec<String>> {
        match meta {
            syn::Meta::NameValue(nv) if nv.path.is_ident("path") => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = &nv.value
                {
                    return self.validate_mod_path(&lit.value(), is_inline);
                }
                self.out.incomplete.push(format!("`{}` contains non-literal `#[path]`", self.file_locator));
            }
            syn::Meta::List(ml) if ml.path.is_ident("cfg_attr") => {
                self.inspect_cfg_attr(ml.tokens.to_string(), is_inline);
            }
            _ => {}
        }
        None
    }

    fn inspect_cfg_attr(&mut self, root_tokens: String, is_inline: bool) {
        let mut queue = Vec::with_capacity(INITIAL_TOKEN_QUEUE_CAP);
        queue.push(root_tokens);
        let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
        while let Some(stream_str) = queue.pop() {
            let Ok(items) = parser.parse_str(&stream_str) else {
                continue;
            };
            for meta in items.into_iter().skip(1) {
                if let syn::Meta::List(ml) = &meta
                    && ml.path.is_ident("cfg_attr")
                {
                    if !push_stream(&mut queue, ml.tokens.to_string(), &mut self.out.incomplete, self.file_locator) {
                        return;
                    }
                } else {
                    let _ = self.inspect_mod_meta(&meta, is_inline);
                }
            }
        }
    }

    fn push_mod_err(&mut self, rel_str: &str, msg: &str) {
        self.out.incomplete.push(format!("`{}` `#[path = \"{rel_str}\"]`: {msg}", self.file_locator));
    }

    fn validate_mod_path(&mut self, rel_str: &str, is_inline: bool) -> Option<Vec<String>> {
        let base = self.mod_dir_stack.last().cloned().unwrap_or_else(|| self.file_base());
        let stack = super::policy::resolve_relative_stack(&base, rel_str)
            .map_err(|err| self.push_mod_err(rel_str, &err))
            .ok()?;
        if stack.first().is_some_and(|first| self.settings.excluded_root_dirs.iter().any(|ex| ex == first)) {
            self.push_mod_err(rel_str, "enters excluded dir");
            return None;
        }
        let check = if is_inline {
            super::policy::check_existing_prefix_no_symlinks(self.package_root, &stack)
        } else {
            super::policy::check_relative_segments(self.package_root, &stack, true).map(|_| ())
        };
        if let Err(err) = check {
            self.push_mod_err(rel_str, &err);
            return None;
        }
        if !is_inline {
            let loc = stack.join("/");
            let is_ext_allowed = Path::new(&loc)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| self.settings.included_extensions.iter().any(|a| a == e));
            if !is_ext_allowed {
                self.push_mod_err(rel_str, "outside included_extensions");
                return None;
            }
            self.out.module_locators.push(loc);
        }
        Some(stack)
    }

    fn file_base(&self) -> Vec<String> {
        let parent = Path::new(self.file_locator).parent().unwrap_or_else(|| Path::new(""));
        parent.components().filter_map(|c| c.as_os_str().to_str().map(str::to_owned)).collect()
    }

    fn handle_include_macro(&mut self, macro_name: &str, tokens_str: &str) {
        let Ok(lit) = syn::parse_str::<syn::LitStr>(tokens_str) else {
            return self.out.unsupported.push(format!("`{}` contains non-literal `{macro_name}!`", self.file_locator));
        };
        let rel = lit.value();
        let res = super::policy::resolve_relative_stack(&self.file_base(), &rel).and_then(|stack| {
            let loc = stack.join("/");
            let path = super::policy::check_relative_segments(self.package_root, &stack, true)?;
            super::read_regular_non_symlink_file(&path).map(|bytes| (loc, bytes))
        });
        match res {
            Ok((loc, bytes)) => drop(self.out.generated.insert(loc, bytes)),
            Err(err) => self.out.incomplete.push(format!("`{}` `{macro_name}!(\"{rel}\")`: {err}", self.file_locator)),
        }
    }
}

fn push_stream(q: &mut Vec<String>, s: String, inc: &mut Vec<String>, loc: &str) -> bool {
    if q.len() >= MAX_TOKEN_QUEUE {
        inc.push(format!("`{loc}` macro token nesting exceeded {MAX_TOKEN_QUEUE}"));
        return false;
    }
    q.push(s);
    true
}

fn parse_delimited_string(input: syn::parse::ParseStream<'_>) -> syn::Result<Option<String>> {
    let inner;
    if input.peek(syn::token::Paren) {
        let _ = syn::parenthesized!(inner in input);
    } else if input.peek(syn::token::Bracket) {
        let _ = syn::bracketed!(inner in input);
    } else if input.peek(syn::token::Brace) {
        let _ = syn::braced!(inner in input);
    } else {
        return Ok(None);
    }
    Ok(Some(inner.cursor().token_stream().to_string()))
}

fn tokens_follow_mod_kind(input: &syn::parse::ParseBuffer<'_>) -> syn::Result<Option<bool>> {
    while !input.is_empty() {
        if input.peek(syn::Token![#]) && input.peek2(syn::token::Bracket) {
            let _pound: syn::Token![#] = input.parse()?;
            let _ = parse_delimited_string(input)?;
        } else if input.peek(syn::Token![pub]) {
            let _pub: syn::Token![pub] = input.parse()?;
            if input.peek(syn::token::Paren) {
                let _ = parse_delimited_string(input)?;
            }
        } else if input.peek(syn::Token![mod]) {
            let _mod: syn::Token![mod] = input.parse()?;
            let _ident: syn::Ident = input.call(syn::Ident::parse_any)?;
            return Ok(Some(input.peek(syn::token::Brace)));
        } else {
            return Ok(None);
        }
    }
    Ok(None)
}
