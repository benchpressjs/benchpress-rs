extern crate benchpress_sys;

#[macro_use]
extern crate neon;

use neon::vm::{Call, JsResult};
use neon::js::error::{JsError, Kind};
use neon::js::{JsString, Value};

fn compile_source(call: Call) -> JsResult<JsString> {
    let scope = call.scope;
    match call.arguments.get(scope, 0) {
        Some(val) => {
            let code = benchpress_sys::compile(val.to_string(scope)?.value().as_str());
            match JsString::new(scope, code.as_ref()) {
                Some(ret) => Ok(ret),
                None => JsError::throw(Kind::SyntaxError, "failed to build a JS String"),
            }
        },
        None => JsError::throw(Kind::TypeError, "not enough arguments"),
    }
}

register_module!(m, {
    m.export("compile", compile_source)
});
