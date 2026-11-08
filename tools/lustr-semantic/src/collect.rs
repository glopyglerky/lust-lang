//! Owned inspection data only. No TyCtxt, Ty, Instance, or MIR reference escapes
//! the rustc callback, and no record is an executable serialization of MIR.
use rustc_abi::{FieldsShape, TagEncoding, Variants};
use rustc_middle::mir::{TerminatorKind, UnwindAction};
use rustc_middle::mono::MonoItem;
use rustc_middle::ty::layout::TyAndLayout;
use rustc_middle::ty::{self, EarlyBinder, Instance, InstanceKind, ShimKind, Ty, TyCtxt, TypingEnv};
use rustc_span::def_id::LOCAL_CRATE;
use rustc_target::callconv::{ArgAbi, ArgAttributes, PassMode};
use serde_json::{Value, json};

pub fn report(tcx: TyCtxt<'_>) -> Result<Value, String> {
    // `cargo check` intentionally omits monomorphization. Forcing it here could
    // reject programs that the requested check accepts, so leave that mode alone.
    let codegen = tcx.sess.opts.output_types.should_codegen() && !tcx.sess.opts.unstable_opts.no_codegen;
    let mut units = Vec::new();
    if codegen {
        for unit in tcx.collect_and_partition_mono_items(()).codegen_units {
            let mut items = Vec::new();
            for (item, data) in unit.items() {
                let mut entry = match *item {
                    MonoItem::Fn(instance) => function(tcx, instance)?,
                    MonoItem::Static(def) => {
                        let ty = tcx.normalize_erasing_regions(TypingEnv::fully_monomorphized(), tcx.type_of(def).instantiate_identity());
                        json!({"kind": "static", "definition": tcx.def_path_str(def), "thread_local": tcx.is_thread_local_static(def), "layout": type_layout(tcx, ty)?})
                    }
                    MonoItem::GlobalAsm(_) => json!({"kind": "global_asm", "lowering_supported": false}),
                };
                entry["symbol"] = json!(item.symbol_name(tcx).name);
                entry["linkage"] = json!(format!("{:?}", data.linkage));
                entry["visibility"] = json!(format!("{:?}", data.visibility));
                entry["local_copy"] = json!(data.inlined);
                items.push(entry);
            }
            items.sort_by(|left, right| left["symbol"].as_str().cmp(&right["symbol"].as_str()));
            units.push(json!({"name": unit.name().as_str(), "items": items}));
        }
        units.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    }
    let mut dependencies = tcx
        .crates(())
        .iter()
        .map(|&krate| {
            json!({
                "name": tcx.crate_name(krate).as_str(),
                "stable_id": format!("{:?}", tcx.stable_crate_id(krate)),
                "crate_hash": format!("{:?}", tcx.crate_hash(krate)),
            })
        })
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left["stable_id"].as_str().cmp(&right["stable_id"].as_str()));
    let sources = tcx
        .sess
        .source_map()
        .files()
        .iter()
        .map(|source| {
            json!({
                "name": source.name.prefer_remapped_unconditionally().to_string(),
                "compiler_source_hash": source.src_hash.to_string(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schema": crate::contract::SCHEMA,
        "schema_version": crate::contract::VERSION,
        "purpose": "inspection-only",
        "executable_ir": false,
        "cache_key_complete": false,
        "compilation_succeeded": false,
        "crate": {
            "name": tcx.crate_name(LOCAL_CRATE).as_str(),
            "stable_id": format!("{:?}", tcx.stable_crate_id(LOCAL_CRATE)),
            // The local crate hash is only available after metadata encoding.
            // Do not force encoding early or invent a complete artifact key.
            "crate_hash": null,
            "crate_hash_status": "not-available-after-analysis",
            "types": tcx.crate_types().iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        },
        "configuration": {
            "target": tcx.sess.opts.target_triple.to_string(),
            "data_layout": tcx.sess.target.data_layout.as_ref(),
            "panic_strategy": format!("{:?}", tcx.sess.panic_strategy()),
            "opt_level": format!("{:?}", tcx.sess.opts.optimize),
            "debug_assertions": tcx.sess.opts.debug_assertions,
            "overflow_checks": tcx.sess.overflow_checks(),
            "mode": if codegen { "codegen" } else { "analysis-only" },
        },
        "sources": sources,
        "dependencies": dependencies,
        "codegen_units": units,
    }))
}

fn function<'tcx>(tcx: TyCtxt<'tcx>, instance: Instance<'tcx>) -> Result<Value, String> {
    let typing_env = TypingEnv::fully_monomorphized();
    let abi = tcx
        .fn_abi_of_instance(typing_env.as_query_input((instance, ty::List::empty())))
        .map_err(|error| format!("function ABI for {instance}: {error:?}"))?;
    let body = tcx.instance_mir(instance.def);
    let mut locals = Vec::new();
    for (index, local) in body.local_decls.iter_enumerated() {
        let ty = instance.instantiate_mir_and_normalize_erasing_regions(tcx, typing_env, EarlyBinder::bind(tcx, local.ty));
        locals.push(json!({"index": index.index(), "layout": type_layout(tcx, ty)?, "needs_drop": ty.needs_drop(tcx, typing_env)}));
    }
    let mut blocks = Vec::new();
    for (index, block) in body.basic_blocks.iter_enumerated() {
        let terminator = block.terminator();
        let mut value = json!({
            "index": index.index(),
            "cleanup": block.is_cleanup,
            "statement_count": block.statements.len(),
            "terminator": terminator_name(&terminator.kind),
            "successors": terminator.successors().map(|target| target.index()).collect::<Vec<_>>(),
            "unwind": terminator.unwind().map(unwind),
        });
        if let TerminatorKind::Drop { place, target, .. } = &terminator.kind {
            let ty = instance.instantiate_mir_and_normalize_erasing_regions(
                tcx,
                typing_env,
                EarlyBinder::bind(tcx, place.ty(&body.local_decls, tcx).ty),
            );
            value["drop"] = json!({"place_debug": format!("{place:?}"), "type": ty.to_string(), "normal_target": target.index()});
        }
        blocks.push(value);
    }
    Ok(json!({
        "kind": "function",
        "definition": tcx.def_path_str(instance.def_id()),
        "instance": instance.to_string(),
        "instance_kind_debug": format!("{:?}", instance.def),
        "generic_arguments_debug": format!("{:?}", instance.args),
        "drop_glue": matches!(instance.def, InstanceKind::Shim(ShimKind::DropGlue(..))),
        "abi": {
            "calling_convention": format!("{:?}", abi.conv),
            "can_unwind": abi.can_unwind,
            "variadic": abi.c_variadic,
            "fixed_argument_count": abi.fixed_count,
            "return": argument(tcx, &abi.ret)?,
            "arguments": abi.args.iter().map(|argument_abi| argument(tcx, argument_abi)).collect::<Result<Vec<_>, _>>()?,
        },
        "mir": {"phase": format!("{:?}", body.phase), "argument_count": body.arg_count, "locals": locals, "blocks": blocks},
    }))
}

fn argument<'tcx>(tcx: TyCtxt<'tcx>, arg: &ArgAbi<'tcx, Ty<'tcx>>) -> Result<Value, String> {
    Ok(json!({"layout": layout_value(tcx, arg.layout), "pass": pass_mode(&arg.mode)}))
}

fn attributes(attrs: &ArgAttributes) -> Value {
    json!({
        "flags": attrs.regular.bits(),
        "extension": format!("{:?}", attrs.arg_ext),
        "pointee_size_bytes": attrs.pointee_size.bytes(),
        "pointee_align_bytes": attrs.pointee_align.map(|align| align.bytes()),
    })
}

fn pass_mode(mode: &PassMode) -> Value {
    match mode {
        PassMode::Ignore => json!({"kind": "ignore"}),
        PassMode::Direct(attrs) => json!({"kind": "direct", "attributes": attributes(attrs)}),
        PassMode::Pair(first, second) => json!({"kind": "pair", "first": attributes(first), "second": attributes(second)}),
        PassMode::Cast { pad_i32_count, cast } => json!({
            "kind": "cast", "pad_i32_count": pad_i32_count,
            "register_prefix": cast.prefix.iter().map(|reg| json!({"kind": format!("{:?}", reg.kind), "size_bytes": reg.size.bytes()})).collect::<Vec<_>>(),
            "rest_offset_bytes": cast.rest_offset.map(|size| size.bytes()),
            "rest": {"kind": format!("{:?}", cast.rest.unit.kind), "unit_size_bytes": cast.rest.unit.size.bytes(), "total_size_bytes": cast.rest.total.bytes(), "consecutive": cast.rest.is_consecutive},
            "attributes": attributes(&cast.attrs),
        }),
        PassMode::Indirect { attrs, address_space, mode } => json!({
            "kind": "indirect", "attributes": attributes(attrs), "address_space_debug": format!("{address_space:?}"), "mode": format!("{mode:?}")}),
        PassMode::IndirectUnsized { attrs, meta_attrs } => {
            json!({"kind": "indirect-unsized", "attributes": attributes(attrs), "metadata_attributes": attributes(meta_attrs)})
        }
    }
}

fn type_layout<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Result<Value, String> {
    let layout =
        tcx.layout_of(TypingEnv::fully_monomorphized().as_query_input(ty)).map_err(|error| format!("layout of {ty}: {error:?}"))?;
    Ok(layout_value(tcx, layout))
}

fn layout_value<'tcx>(tcx: TyCtxt<'tcx>, layout: TyAndLayout<'tcx>) -> Value {
    let fields = match &layout.fields {
        FieldsShape::Primitive => json!({"kind": "primitive"}),
        FieldsShape::Union(count) => json!({"kind": "union", "count": count.get()}),
        FieldsShape::Array { stride, count } => json!({"kind": "array", "stride_bytes": stride.bytes(), "count": count}),
        FieldsShape::Arbitrary { offsets, in_memory_order } => json!({
            "kind": "arbitrary", "offsets_bytes": offsets.iter().map(|size| size.bytes()).collect::<Vec<_>>(),
            "memory_order": in_memory_order.iter().map(|index| index.index()).collect::<Vec<_>>(),
        }),
    };
    let variants = match &layout.variants {
        Variants::Empty => json!({"kind": "empty"}),
        Variants::Single { index } => json!({"kind": "single", "index": index.index()}),
        Variants::Multiple { tag, tag_encoding, tag_field, variants } => json!({
            "kind": "multiple", "tag_debug": format!("{tag:?}"), "tag_field": tag_field.index(),
            "encoding": match tag_encoding {
                TagEncoding::Direct => json!({"kind": "direct"}),
                TagEncoding::Niche { untagged_variant, niche_variants, niche_start } => json!({
                    "kind": "niche", "untagged_variant": untagged_variant.index(),
                    "variants_start": niche_variants.start.index(), "variants_end": niche_variants.last.index(),
                    "niche_start_decimal": niche_start.to_string(),
                }),
            },
            "variants": variants.iter_enumerated().map(|(index, variant)| json!({
                "index": index.index(), "size_bytes": variant.size.bytes(),
                "field_offsets_bytes": variant.field_offsets.iter().map(|size| size.bytes()).collect::<Vec<_>>(),
                "backend_repr_debug": format!("{:?}", variant.backend_repr), "uninhabited": variant.is_uninhabited(),
            })).collect::<Vec<_>>(),
        }),
    };
    json!({
        "type": layout.ty.to_string(), "type_id_debug": format!("{:?}", tcx.type_id_hash(layout.ty)),
        "size_bytes": layout.size.bytes(), "align_bytes": layout.align.bytes(), "sized": layout.is_sized(),
        "uninhabited": layout.uninhabited, "backend_repr_debug": format!("{:?}", layout.backend_repr),
        "fields": fields, "variants": variants,
        "largest_niche_debug": format!("{:?}", layout.largest_niche),
    })
}

fn unwind(action: &UnwindAction) -> Value {
    match action {
        UnwindAction::Continue => json!({"kind": "continue"}),
        UnwindAction::Unreachable => json!({"kind": "unreachable"}),
        UnwindAction::Terminate(reason) => json!({"kind": "terminate", "reason": format!("{reason:?}")}),
        UnwindAction::Cleanup(target) => json!({"kind": "cleanup", "target": target.index()}),
    }
}

fn terminator_name(kind: &TerminatorKind<'_>) -> &'static str {
    match kind {
        TerminatorKind::Goto { .. } => "goto",
        TerminatorKind::SwitchInt { .. } => "switch-int",
        TerminatorKind::UnwindResume => "unwind-resume",
        TerminatorKind::UnwindTerminate(_) => "unwind-terminate",
        TerminatorKind::Return => "return",
        TerminatorKind::Unreachable => "unreachable",
        TerminatorKind::Drop { .. } => "drop",
        TerminatorKind::Call { .. } => "call",
        TerminatorKind::TailCall { .. } => "tail-call",
        TerminatorKind::Assert { .. } => "assert",
        TerminatorKind::Yield { .. } => "yield",
        TerminatorKind::CoroutineDrop => "coroutine-drop",
        TerminatorKind::FalseEdge { .. } => "false-edge",
        TerminatorKind::FalseUnwind { .. } => "false-unwind",
        TerminatorKind::InlineAsm { .. } => "inline-asm",
    }
}
