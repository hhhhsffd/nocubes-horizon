use quote::{ToTokens, quote};
use std::{
    fs,
    path::{Path, PathBuf},
};
use syn::{
    FnArg, GenericParam, ImplItem, Item, Pat, ReturnType, Type,
    visit_mut::{self, VisitMut},
};
fn tokens(t: &impl ToTokens) -> String {
    t.to_token_stream().to_string()
}
fn inactive(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        let t = tokens(a);
        t.contains("cfg (test)") || t.contains("cfg (any (")
    })
}
#[derive(Default)]
struct Rewrite {
    count: usize,
}
impl VisitMut for Rewrite {
    fn visit_expr_method_call_mut(&mut self, n: &mut syn::ExprMethodCall) {
        if (n.method == "collision_shape" && n.args.len() == 3)
            || (n.method == "collision_shape_with_context" && n.args.len() == 4)
        {
            n.method = syn::Ident::new(&format!("nocubes_{}", n.method), n.method.span());
            self.count += 1;
        }
        visit_mut::visit_expr_method_call_mut(self, n);
    }
}
#[derive(Default)]
struct ArrowMethods {
    count: usize,
}
impl VisitMut for ArrowMethods {
    fn visit_expr_method_call_mut(&mut self, n: &mut syn::ExprMethodCall) {
        if matches!(
            n.method.to_string().as_str(),
            "apply_block_effects" | "check_inside_blocks"
        ) {
            n.method = syn::Ident::new(&format!("nocubes_{}", n.method), n.method.span());
            self.count += 1;
        }
        visit_mut::visit_expr_method_call_mut(self, n);
    }
}
struct ArrowFluid {
    count: usize,
}
impl VisitMut for ArrowFluid {
    fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
        if let syn::Expr::MethodCall(c) = e
            && c.method == "fluid"
            && tokens(&c.receiver).ends_with("fluids")
        {
            let scene = if tokens(&c.receiver).contains("scene") {
                quote!(&self.scene)
            } else {
                quote!(self)
            };
            let a = c.args.first().unwrap();
            let pos = if let syn::Expr::MethodCall(b) = a {
                b.args.first().unwrap().clone()
            } else if tokens(a) == "above" {
                syn::parse_quote!(BlockPos::new(p.x, p.y + 1, p.z))
            } else {
                syn::parse_quote!(p)
            };
            *e = syn::parse2(quote!(nocubes_arrow_fluid(#scene,#pos))).unwrap();
            self.count += 1;
            return;
        }
        visit_mut::visit_expr_mut(self, e);
    }
}
struct EntityFluid {
    count: usize,
}
impl VisitMut for EntityFluid {
    fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
        if let syn::Expr::MethodCall(c) = e
            && c.method == "fluid"
            && tokens(&c.receiver).ends_with("fluids")
            && let Some(syn::Expr::MethodCall(b)) = c.args.first()
            && b.method == "block"
            && b.args.len() == 3
        {
            let coords = b.args.iter();
            let (world, fluids) = if tokens(&c.receiver).starts_with("self") {
                (quote!(&self.world), quote!(&self.fluids))
            } else {
                (quote!(world), quote!(fluids))
            };
            *e = syn::parse2(quote!(nocubes_entity_fluid(#world,#fluids,[#(#coords),*]))).unwrap();
            self.count += 1;
            return;
        }
        visit_mut::visit_expr_mut(self, e);
    }
}
struct Nonplayer {
    count: usize,
}
impl VisitMut for Nonplayer {
    fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
        let wrap = match &e {
            syn::Expr::MethodCall(c) => matches!(
                c.method.to_string().as_str(),
                "resolve"
                    | "resolve_in_border"
                    | "resolve_with_context"
                    | "nocubes_collision_shape"
                    | "nocubes_collision_shape_with_context"
            ),
            syn::Expr::Call(c) => {
                matches!(&*c.func,syn::Expr::Path(p) if p.path.segments.last().is_some_and(|s|matches!(s.ident.to_string().as_str(),"no_world_collision"|"resolve_world_movement")))
            }
            _ => false,
        };
        visit_mut::visit_expr_mut(self, e);
        if wrap {
            let inner = e.clone();
            *e = syn::parse_quote!(horizon_client::collisions::nocubes_collision::with_nonplayer(|| #inner));
            self.count += 1;
        }
    }
}
struct Lifetime;
impl VisitMut for Lifetime {
    fn visit_lifetime_mut(&mut self, n: &mut syn::Lifetime) {
        if n.ident != "static" {
            *n = syn::parse_quote!('_);
        }
    }
}
fn normalized(ty: &Type) -> String {
    let mut t = ty.clone();
    Lifetime.visit_type_mut(&mut t);
    tokens(&t).replace(' ', "")
}
fn walk(path: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(path).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let horizon = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/horizon-upstream".into());
    let base = Path::new(&horizon);
    let dest = root.join("mixins/nocubes/generated");
    fs::create_dir_all(&dest).unwrap();
    let mut manifest = String::new();
    let mut report = vec![];
    let mut count = 0;
    for package in ["horizon-client", "horizon-client-render"] {
        let src = base.join("crates").join(package).join("src");
        let mut files = vec![];
        walk(&src, &mut files);
        files.sort();
        for file in files {
            if file.file_name().unwrap() == "lib.rs" || file.file_name().unwrap() == "client_lib.rs"
            {
                continue;
            }
            let relative = file.strip_prefix(base).unwrap().to_str().unwrap();
            let modpath = file
                .strip_prefix(&src)
                .unwrap()
                .with_extension("")
                .to_str()
                .unwrap()
                .replace('/', "::");
            let modpath = modpath.strip_suffix("::mod").unwrap_or(&modpath);
            let actual = if package == "horizon-client"
                && modpath != "game"
                && !modpath.starts_with("game::")
            {
                "horizon_client_lib"
            } else {
                &package.replace('-', "_")
            };
            let module_id = format!("{actual}::{modpath}");
            let ast = syn::parse_file(&fs::read_to_string(&file).unwrap()).unwrap();
            let mut payload = quote!();
            let mut injections = String::new();
            let shadow = if modpath == "game::perspective" {
                Some("VisualRays")
            } else if modpath == "game::explosion" {
                Some("ColliderRays")
            } else if modpath == "mobs::living" {
                Some("FlowScene")
            } else {
                None
            };
            for item in &ast.items {
                match item {
                    Item::Struct(v) if shadow.is_some_and(|n| v.ident == n) => {
                        payload.extend(quote!(#v))
                    }
                    Item::Impl(v) if shadow.is_some_and(|n| tokens(&v.self_ty).starts_with(n)) => {
                        let mut v = v.clone();
                        Rewrite::default().visit_item_impl_mut(&mut v);
                        payload.extend(quote!(#v));
                    }
                    Item::Fn(v) if modpath == "mobs::world" && v.sig.ident == "visual_clip" => {
                        let mut v = v.clone();
                        Rewrite::default().visit_item_fn_mut(&mut v);
                        v.sig.generics.make_where_clause().predicates.push(syn::parse_quote!(W: horizon_client::collisions::nocubes_collision::QueryWorld));
                        payload.extend(quote!(#v));
                    }
                    Item::Impl(v) if modpath == "arrows" => {
                        for item in &v.items {
                            if let ImplItem::Fn(f) = item
                                && matches!(
                                    f.sig.ident.to_string().as_str(),
                                    "apply_block_effects" | "check_inside_blocks"
                                )
                            {
                                let mut f = f.clone();
                                f.sig.ident = syn::Ident::new(
                                    &format!("nocubes_{}", f.sig.ident),
                                    f.sig.ident.span(),
                                );
                                Rewrite::default().visit_impl_item_fn_mut(&mut f);
                                ArrowMethods::default().visit_impl_item_fn_mut(&mut f);
                                ArrowFluid { count: 0 }.visit_impl_item_fn_mut(&mut f);
                                let ty = &v.self_ty;
                                payload.extend(quote!(impl #ty {#f}));
                            }
                        }
                    }
                    Item::Impl(v) if modpath == "game::piston_push" => {
                        for member in &v.items {
                            if let ImplItem::Fn(f) = member
                                && f.sig.ident == "push_for_piston"
                            {
                                let mut f = f.clone();
                                f.sig.ident = syn::parse_quote!(nocubes_push_for_piston);
                                Rewrite::default().visit_impl_item_fn_mut(&mut f);
                                let ty = &v.self_ty;
                                payload.extend(quote!(impl #ty{#f}));
                            }
                        }
                    }
                    _ => {}
                }
            }
            for item in ast.items {
                let (owner, traitpath, generics, functions) = match item {
                    Item::Fn(f) if !inactive(&f.attrs) => (
                        None,
                        None,
                        syn::Generics::default(),
                        vec![(f.attrs, f.sig, *f.block)],
                    ),
                    Item::Impl(i) if !inactive(&i.attrs) => {
                        let funcs = i
                            .items
                            .into_iter()
                            .filter_map(|f| match f {
                                ImplItem::Fn(f) if !inactive(&f.attrs) => {
                                    Some((f.attrs, f.sig, f.block))
                                }
                                _ => None,
                            })
                            .collect();
                        (
                            Some(i.self_ty),
                            i.trait_.map(|(_, p, _)| p),
                            i.generics,
                            funcs,
                        )
                    }
                    _ => continue,
                };
                for (_, mut sig, mut body) in functions {
                    let original = sig.clone();
                    let mut arrow_methods = ArrowMethods::default();
                    if modpath == "arrows" || modpath == "game::trident" {
                        arrow_methods.visit_block_mut(&mut body);
                    }

                    let mut entity_fluid = EntityFluid { count: 0 };
                    if modpath == "entities::fluids" {
                        entity_fluid.visit_block_mut(&mut body);
                    }
                    if modpath == "motion::environment" {
                        struct Bubble;
                        impl VisitMut for Bubble {
                            fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
                                if tokens(e).replace(' ', "")
                                    == "self.resources.fluids.fluid(above)"
                                {
                                    *e = syn::parse_quote!(self.fluid_at(p.offset(0, 1, 0)));
                                    return;
                                }
                                visit_mut::visit_expr_mut(self, e);
                            }
                        }
                        Bubble.visit_block_mut(&mut body);
                    }
                    if modpath == "motion::fluids" && sig.ident == "bubble_column_at" {
                        struct Bubble;
                        impl VisitMut for Bubble {
                            fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
                                if tokens(e).replace(' ', "") == "facts.get(above).fluid" {
                                    *e = syn::parse_quote!({
                                        let cfg = horizon_core::nocubes::config();
                                        horizon_core::nocubes::fluids::extended(
                                            [pos.x, pos.y + 1, pos.z],
                                            cfg.extend_fluids_range,
                                            |q| {
                                                horizon_core::nocubes::kinds::smoothable(
                                                    &cfg,
                                                    world.block(q[0], q[1], q[2]).0,
                                                )
                                            },
                                            |f: (crate::blocks::Fluid, u8, bool)| {
                                                f.0 == crate::blocks::Fluid::None
                                            },
                                            |f: (crate::blocks::Fluid, u8, bool)| f.1 >= 8 && !f.2,
                                            |q| {
                                                let f = facts.get(world.block(q[0], q[1], q[2]));
                                                (f.fluid, f.fluid_level, f.fluid_falling)
                                            },
                                        )
                                        .0
                                    });
                                    return;
                                }
                                visit_mut::visit_expr_mut(self, e);
                            }
                        }
                        Bubble.visit_block_mut(&mut body);
                    }

                    if modpath == "arrows"
                        && matches!(
                            sig.ident.to_string().as_str(),
                            "apply_block_effects" | "check_inside_blocks"
                        )
                    {
                        report.push(format!("SHADOWED_GENERIC {module_id}::{}", sig.ident));
                        continue;
                    }
                    let mut arrow = ArrowFluid { count: 0 };
                    if modpath == "arrows" {
                        arrow.visit_block_mut(&mut body);
                    }

                    let crack_adapter = modpath == "game" && sig.ident == "entity_geometry";
                    let suffocation_adapter = modpath == "game"
                        && matches!(
                            sig.ident.to_string().as_str(),
                            "closest_space_nudge" | "try_sleep"
                        );
                    if suffocation_adapter {
                        struct Suffocation {
                            bed: bool,
                        }
                        impl VisitMut for Suffocation {
                            fn visit_pat_ident_mut(&mut self, p: &mut syn::PatIdent) {
                                if p.ident == "f" {
                                    p.ident = syn::parse_quote!(_f);
                                }
                                visit_mut::visit_pat_ident_mut(self, p);
                            }
                            fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
                                if tokens(e).replace(' ', "") == "f.full_collision&&f.solid_render"
                                {
                                    *e = if self.bed {
                                        syn::parse_quote!(
                                            crate::game::nocubes_suffocation::occupied(
                                                &self.world,
                                                self.res.facts(),
                                                [bed[0], bed[1] + 1, bed[2]],
                                                self.player.as_ref().unwrap().bounds()
                                            )
                                        )
                                    } else {
                                        syn::parse_quote!(
                                            crate::game::nocubes_suffocation::occupied(
                                                world,
                                                facts,
                                                [x, y, z],
                                                body
                                            )
                                        )
                                    };
                                    return;
                                }
                                visit_mut::visit_expr_mut(self, e);
                            }
                        }
                        Suffocation {
                            bed: sig.ident == "try_sleep",
                        }
                        .visit_block_mut(&mut body);
                    }

                    if crack_adapter {
                        struct Cracks;
                        impl VisitMut for Cracks {
                            fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
                                if let syn::Expr::Call(c) = e
                                    && let syn::Expr::Path(p) = &*c.func
                                    && tokens(&p.path).replace(' ', "")
                                        == "horizon_client::breaking::geometry"
                                {
                                    let args = &c.args;
                                    let it = args.iter();
                                    *e = syn::parse2(
                                        quote!(self.nocubes_crack_geometry(pos,#(#it),*)),
                                    )
                                    .unwrap();
                                    return;
                                }
                                visit_mut::visit_expr_mut(self, e);
                            }
                        }
                        Cracks.visit_block_mut(&mut body);
                    }

                    if shadow
                        .is_some_and(|n| owner.as_ref().is_some_and(|t| tokens(t).starts_with(n)))
                    {
                        continue;
                    }
                    let force = shadow.is_some_and(|n| tokens(&body).contains(&format!("{n} {{")))
                        || (modpath == "mobs::world"
                            && owner.is_some()
                            && sig.ident == "visual_clip")
                        || (modpath == "game::piston_push" && sig.ident == "push_piston_entities");
                    let mut rw = Rewrite::default();
                    rw.visit_block_mut(&mut body);
                    let mut nonplayer = Nonplayer { count: 0 };
                    if modpath.starts_with("mobs::") {
                        nonplayer.visit_block_mut(&mut body);
                    }

                    if force && modpath == "game::piston_push" {
                        struct Push;
                        impl VisitMut for Push {
                            fn visit_expr_method_call_mut(&mut self, n: &mut syn::ExprMethodCall) {
                                if n.method == "push_for_piston" {
                                    n.method = syn::parse_quote!(nocubes_push_for_piston);
                                }
                                visit_mut::visit_expr_method_call_mut(self, n);
                            }
                        }
                        Push.visit_block_mut(&mut body);
                    }
                    if rw.count == 0
                        && arrow.count == 0
                        && entity_fluid.count == 0
                        && arrow_methods.count == 0
                        && nonplayer.count == 0
                        && !force
                        && !crack_adapter
                        && !suffocation_adapter
                        && !(modpath == "mobs::living"
                            && matches!(
                                sig.ident.to_string().as_str(),
                                "fluids" | "fluid_currents"
                            ))
                    {
                        continue;
                    }
                    if sig
                        .inputs
                        .iter()
                        .any(|a| matches!(a,FnArg::Typed(a) if !matches!(&*a.pat,Pat::Ident(_))))
                    {
                        report.push(format!(
                            "SHADOWED_OR_DESTRUCTURED {module_id}::{}",
                            sig.ident
                        ));
                        continue;
                    }
                    let target = if let Some(ty) = &owner {
                        let name = normalized(ty);
                        if let Some(tr) = &traitpath {
                            format!(
                                "{module_id}::<{name} as {}>::{}",
                                tokens(tr).replace(' ', ""),
                                sig.ident
                            )
                        } else {
                            format!("{module_id}::{name}::{}", sig.ident)
                        }
                    } else {
                        format!("{module_id}::{}", sig.ident)
                    };
                    if generics
                        .params
                        .iter()
                        .chain(sig.generics.params.iter())
                        .any(|p| !matches!(p, GenericParam::Lifetime(_)))
                        || tokens(&sig.output).contains("Self")
                        || sig.asyncness.is_some()
                    {
                        if modpath == "mobs::world" && sig.ident == "visual_clip" {
                            report.push(format!("SHADOWED_GENERIC {target}"));
                        } else {
                            report.push(format!("UNSUPPORTED {target}"));
                        }
                        continue;
                    }
                    count += 1;
                    let id = format!("query_{count}");
                    let new_name = syn::Ident::new(&format!("nocubes_{id}"), sig.ident.span());
                    sig.ident = new_name.clone();
                    let mut bindings = vec![];
                    let mut params = vec![];
                    let mut arguments = vec![];
                    let mut receiver = false;
                    let mut supported = true;
                    let mut copy_inputs = syn::punctuated::Punctuated::new();
                    for input in &original.inputs {
                        match input {
                            FnArg::Receiver(r) => {
                                if r.reference.is_none() {
                                    supported = false;
                                    break;
                                }
                                let mut ty = *owner.as_ref().unwrap().clone();
                                Lifetime.visit_type_mut(&mut ty);
                                let reference = if r.mutability.is_some() {
                                    quote!(&mut #ty)
                                } else {
                                    quote!(&#ty)
                                };
                                let value = if r.mutability.is_some() {
                                    "&mut *self"
                                } else {
                                    "&*self"
                                };
                                bindings.push(serde_json::json!({"name":"receiver","ty":reference.to_string(),"value":value}));
                                params.push(quote!(receiver:#reference));
                                receiver = true;
                                copy_inputs.push(input.clone());
                            }
                            FnArg::Typed(a) => {
                                if matches!(&*a.pat, Pat::Wild(_)) {
                                    continue;
                                }
                                let Pat::Ident(p) = &*a.pat else {
                                    supported = false;
                                    break;
                                };
                                let name = &p.ident;
                                let mut ty = *a.ty.clone();
                                Lifetime.visit_type_mut(&mut ty);
                                let vector = matches!(&ty,Type::Path(p) if p.path.segments.last().is_some_and(|s|s.ident=="Vec"));
                                let mutable_vector = vector && p.mutability.is_some();
                                let value = if mutable_vector {
                                    format!("&mut {name}")
                                } else if vector {
                                    format!("&{name}")
                                } else if let Type::Reference(r) = &ty {
                                    if r.mutability.is_some() {
                                        format!("&mut *{name}")
                                    } else {
                                        format!("&*{name}")
                                    }
                                } else {
                                    format!("{name}")
                                };
                                let binding_ty = if mutable_vector {
                                    quote!(&mut #ty)
                                } else if vector {
                                    quote!(&#ty)
                                } else {
                                    quote!(#ty)
                                };
                                bindings.push(serde_json::json!({"name":name.to_string(),"ty":binding_ty.to_string(),"value":value}));
                                params.push(quote!(#name:#binding_ty));
                                arguments.push(if mutable_vector {
                                    quote!(std::mem::take(#name))
                                } else if vector {
                                    quote!(#name.clone())
                                } else {
                                    quote!(#name)
                                });
                                copy_inputs.push(input.clone());
                            }
                        }
                    }
                    if !supported {
                        report.push(format!("UNSUPPORTED_PARAMETERS {target}"));
                        continue;
                    }
                    sig.inputs = copy_inputs;
                    let body = quote!(#body);
                    let ret = match &original.output {
                        ReturnType::Default => quote!(()),
                        ReturnType::Type(_, t) => quote!(#t),
                    };
                    let reference_owner = owner
                        .as_ref()
                        .is_some_and(|t| matches!(&**t, Type::Reference(_)));
                    let ext = syn::Ident::new(&format!("NocubesQuery{count}"), sig.ident.span());
                    let call = if receiver && reference_owner {
                        quote!(nocubes_queries::#ext::#new_name(receiver,#(#arguments),*))
                    } else if receiver {
                        quote!(receiver.#new_name(#(#arguments),*))
                    } else {
                        quote!(nocubes_queries::#new_name(#(#arguments),*))
                    };
                    if let Some(ty) = &owner {
                        if reference_owner {
                            payload.extend(quote!(pub(super) trait #ext {#sig;} impl #generics #ext for #ty {#sig #body}));
                        } else {
                            payload.extend(quote!(impl #generics #ty { pub(super) #sig #body }));
                        }
                    } else {
                        payload.extend(quote!(pub(super) #sig #body));
                    }
                    let helper = quote!(fn adapter(#(#params),*)->core::ops::ControlFlow<#ret> { core::ops::ControlFlow::Break(#call) });
                    fs::write(dest.join(format!("{id}.rs")), helper.to_string()).unwrap();
                    injections += &format!(
                        "\n[[injection]]\nid = \"{id}\"\ntarget = {}\nkind = \"HEAD\"\ncancellable = true\nhelper = \"generated/{id}.rs\"\nbindings = [\n{}\n]\n",
                        serde_json::to_string(&target).unwrap(),
                        bindings
                            .iter()
                            .map(|b| format!(
                                "{{name={},ty={},value={}}}",
                                b["name"], b["ty"], b["value"]
                            ))
                            .collect::<Vec<_>>()
                            .join(",\n")
                    );
                    report.push(format!(
                        "ADAPTED {target} ({} collision/helper calls, {} fluid reads)",
                        rw.count,
                        arrow.count + entity_fluid.count
                    ));
                }
            }
            if !injections.is_empty() {
                struct ShiftSuper;
                impl VisitMut for ShiftSuper {
                    fn visit_visibility_mut(&mut self, _: &mut syn::Visibility) {}
                    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
                        fn shift(stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
                            use proc_macro2::{Group, TokenTree};
                            let items: Vec<_> = stream.into_iter().collect();
                            let mut out = proc_macro2::TokenStream::new();
                            for (i, item) in items.iter().enumerate() {
                                if matches!(item,TokenTree::Ident(id) if id=="super")
                                    && matches!(items.get(i+1),Some(TokenTree::Punct(p)) if p.as_char()==':')
                                    && matches!(items.get(i+2),Some(TokenTree::Punct(p)) if p.as_char()==':')
                                    && !matches!(i.checked_sub(1).and_then(|i|items.get(i)),Some(TokenTree::Punct(p)) if p.as_char()==':')
                                {
                                    out.extend(quote!(super::));
                                }
                                out.extend([match item {
                                    TokenTree::Group(g) => {
                                        let mut group =
                                            Group::new(g.delimiter(), shift(g.stream()));
                                        group.set_span(g.span());
                                        TokenTree::Group(group)
                                    }
                                    other => other.clone(),
                                }]);
                            }
                            out
                        }
                        mac.tokens = shift(mac.tokens.clone());
                    }

                    fn visit_path_mut(&mut self, path: &mut syn::Path) {
                        if path.segments.first().is_some_and(|s| s.ident == "super") {
                            path.segments.insert(0, syn::parse_quote!(super));
                        }
                        visit_mut::visit_path_mut(self, path);
                    }
                }
                struct FluidReads;
                impl VisitMut for FluidReads {
                    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
                        if let syn::Expr::MethodCall(call) = expr {
                            if call.method == "fluid"
                                && tokens(&call.receiver).ends_with("fluid_tables")
                                && call.args.len() == 1
                                && let Some(syn::Expr::MethodCall(block)) = call.args.first()
                                && block.method == "block"
                                && block.args.len() == 3
                            {
                                let p = &block.args;
                                let coords = p.iter();
                                let receiver = if tokens(&call.receiver).starts_with("self . 0") {
                                    quote!(self.0)
                                } else {
                                    quote!(self)
                                };
                                *expr =
                                    syn::parse2(quote!(nocubes_extended(#receiver,[#(#coords),*])))
                                        .unwrap();
                                return;
                            }
                            if call.method == "may_hold" {
                                let original = call.clone();
                                *expr=syn::parse2(quote!((horizon_core::nocubes::config().extend_fluids_range>0 || #original))).unwrap();
                                return;
                            }
                        }
                        visit_mut::visit_expr_mut(self, expr);
                    }
                }
                let mut parsed: syn::File = syn::parse2(payload).unwrap();
                if modpath == "mobs::living" {
                    FluidReads.visit_file_mut(&mut parsed);
                    let extra: syn::ItemFn = syn::parse_quote! {
                        fn nocubes_extended(world:&MobWorld<'_>,p:[i32;3])->FluidState {
                            let cfg=horizon_core::nocubes::config();
                            horizon_core::nocubes::fluids::extended(p,cfg.extend_fluids_range,|p|horizon_core::nocubes::kinds::smoothable(&cfg,world.world.block(p[0],p[1],p[2]).0),|f:FluidState|f.kind()==FluidKind::Empty,|f:FluidState|f.is_source(),|p|world.fluid_tables.fluid(world.world.block(p[0],p[1],p[2])))
                        }
                    };
                    parsed.items.push(syn::Item::Fn(extra));
                }
                if modpath == "arrows" {
                    parsed.items.push(syn::parse_quote! {
                        fn nocubes_arrow_fluid(scene:&ArrowScene<'_,'_>,p:BlockPos)->FluidState {
                            let cfg=horizon_core::nocubes::config();
                            horizon_core::nocubes::fluids::extended([p.x,p.y,p.z],cfg.extend_fluids_range,|q|horizon_core::nocubes::kinds::smoothable(&cfg,scene.block(BlockPos::new(q[0],q[1],q[2])).0),|f:FluidState|f.kind()==FluidKind::Empty,|f:FluidState|f.is_source(),|q|scene.fluids.fluid(scene.block(BlockPos::new(q[0],q[1],q[2]))))
                        }
                    });
                }
                if modpath == "entities::fluids" {
                    parsed.items.push(syn::parse_quote! {
                        fn nocubes_entity_fluid(world:&World,fluids:&FluidResources,p:[i32;3])->FluidState {
                            let cfg=horizon_core::nocubes::config();
                            horizon_core::nocubes::fluids::extended(p,cfg.extend_fluids_range,|q|horizon_core::nocubes::kinds::smoothable(&cfg,world.block(q[0],q[1],q[2]).0),|f:FluidState|f.kind()==FluidKind::Empty,|f:FluidState|f.is_source(),|q|fluids.fluid(world.block(q[0],q[1],q[2])))
                        }
                    });
                }
                ShiftSuper.visit_file_mut(&mut parsed);
                let payload = quote!(#parsed);

                let name = format!("queries_{}", modpath.replace("::", "_"));
                let import = if !tokens(&payload).contains("nocubes_collision_shape") {
                    quote!()
                } else if package == "horizon-client" {
                    quote!(
                        use horizon_client::collisions::nocubes_collision::RegistryExt;
                    )
                } else {
                    quote!(
                        use crate::collisions::nocubes_collision::RegistryExt;
                    )
                };
                fs::write(
                    dest.join(format!("{name}.rs")),
                    quote!(use super::*;#import #payload).to_string(),
                )
                .unwrap();
                // The helper references this common module name for free functions.
                manifest += &format!(
                    "\n[[module]]\nid=\"{name}\"\nfile={}\nname=\"nocubes_queries\"\nsource=\"generated/{name}.rs\"\n",
                    serde_json::to_string(relative).unwrap()
                );
                manifest += &injections;
            }
        }
    }
    fs::write(
        root.join("validation/query-adapters.txt"),
        report.join("\n"),
    )
    .unwrap();
    fs::write(root.join("tools/query-adapters.toml"), &manifest).unwrap();
    let path = root.join("mixins/nocubes/horizon-mixin.toml");
    let current = fs::read_to_string(&path).unwrap();
    let manual = current.split("# Generated query adapters").next().unwrap();
    fs::write(
        path,
        format!("{manual}# Generated query adapters\n{manifest}"),
    )
    .unwrap();
    println!("generated {count} query adapters");
}
