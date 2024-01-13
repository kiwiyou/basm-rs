if __name__ == '__main__':
    from parse import Signature
else:
    from .parse import Signature

def emit_export(sig: Signature, offset: int):
    args_in_cpp_syntax = ", ".join(
        ["{ty} {ident}".format(ty = ty, ident = ident) for (ident, ty) in sig.args]
    )
    template = "\n".join([
        r"{output} {ident}({args_in_cpp_syntax}) {{",
        r"    // TODO",
        r"}}"
    ]) + "\n"
    out = template.format(
        args_in_cpp_syntax = args_in_cpp_syntax,
        ident = sig.ident,
        output = sig.output
    )
    return out, None

def emit_import(sig: Signature, offset: int):
    arg_de = "\n".join(
        [r"    {ty} arg{id} = do_de<{ty}>(ptr_serialized);".format(ty = ty, id = i)
            for (i, (_ident, ty)) in enumerate(sig.args)]
    )
    arg_names = ", ".join(
        ["arg{id}".format(id = i) for i in range(len(sig.args))]
    )
    template = "\n".join([
        r"BASMCALL size_t basm_import_thunk_{ident}(size_t ptr_serialized) {{",
        r"    static std::vector<uint8_t> s_buf;",
        r"    struct basm_free_impl {{",
        r"        static void free() {{",
        r"            s_buf.clear();",
        r"        }}",
        r"    }};",
        r"",
        r"    do_de<size_t>(ptr_serialized);",
        arg_de,
        r"    ((void (BASMCALL *)()) do_de<size_t>(ptr_serialized))();",
        r"",
        r"    {output} out = {ident}({arg_names});",
        r"    do_ser<size_t>(s_buf, 0);",
        r"    do_ser<{output}>(s_buf, out);",
        r"    do_ser<size_t>(s_buf, (size_t) basm_free_impl::free);",
        r"    do_ser_end(s_buf);",
        r"    return (size_t) s_buf.data();",
        r"}}",
    ]) + "\n"
    out = template.format(
        ident = sig.ident,
        arg_de = arg_de,
        arg_names = arg_names,
        output = sig.output
    )
    on_loaded = "\n".join([
        r"    ((void (BASMCALL *)(size_t))(BASM_LOADER_IMAGEBASE + {offset}))((size_t) basm_import_thunk_{ident});"
    ]).format(
        offset = offset,
        ident = sig.ident
    )
    return out, on_loaded

def emit(sig: Signature, offset: int):
    assert type(sig) == Signature
    if sig.bindgen_type == Signature.EXPORT:
        return emit_export(sig, offset)
    else:
        assert sig.bindgen_type == Signature.IMPORT
        return emit_import(sig, offset)

def emit_all(sig_offset_list):
    bodies, on_loaded_clauses = [], []
    for sig, offset in sig_offset_list:
        body, on_loaded = emit(sig, offset)
        bodies.append(body)
        if on_loaded:
            on_loaded_clauses.append(on_loaded)
    body = "\n".join(bodies) + "\n"
    on_loaded = "\n".join([
        r"void basm_on_loaded() {",
    ] + on_loaded_clauses + [
        r"}"
    ]) + "\n"
    return body + on_loaded

if __name__ == '__main__':
    sig_offset_list = [
        (Signature("_basm_export_4_init_2_1_t_prim_i32_1_n_prim_i32_prim_unit"), 2083),
        (Signature("_basm_export_4_game_0_prim_unit"), 1142),
        (Signature("_basm_import_5_guess_1_1_b_prim_string_pair_prim_i32_prim_i32"), 648),
        (Signature("_basm_import_8_test_ptr_2_1_x_prim_ptr_usize_1_y_vec_pair_prim_i8_prim_u64_prim_ptrmut_u8"), 94576)
    ]
    print(emit_all(sig_offset_list))