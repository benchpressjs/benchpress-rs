use neon::prelude::*;

fn compile_source(mut cx: FunctionContext) -> JsResult<JsString> {
    let val = cx.argument::<JsString>(0)?;

    let code = benchpress_sys::compile(val.to_string(&mut cx)?.value().as_str());
    Ok(cx.string(&code))
}

register_module!(mut cx, {
    cx.export_function("compile", compile_source)?;
    Ok(())
});
