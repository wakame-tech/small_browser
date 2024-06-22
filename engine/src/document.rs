use boa_engine::{
    class::Class, js_string, Context, Finalize, JsData, JsResult, JsValue, NativeFunction, Trace,
};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct Document;

impl Document {
    fn get_element_by_id(
        _this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let id = args[0].as_string().unwrap().to_std_string_escaped();
        // 現在描画されている DOM ツリーの Rust 上での表現を特定する
        // let document_element = JavaScriptRuntime::document_element();
        // let document_element = &mut document_element.borrow_mut();

        // // `getElementById()` の引数を用いて DOM ツリーを検索し、返り値を設定する
        // let Some(element) = document_element.get_element_by_id(id.as_str()) else {
        //     return Err(JsError::from_opaque(JsValue::String(js_string!(
        //         "element not found"
        //     ))));
        // };
        Ok(JsValue::Null)
    }
}

impl Class for Document {
    const NAME: &'static str = "Document";

    fn data_constructor(
        new_target: &boa_engine::JsValue,
        args: &[boa_engine::JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(Document)
    }

    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        // `getElementById()` 関数の定義
        let get_element_by_id = NativeFunction::from_fn_ptr(Self::get_element_by_id);
        class.method(js_string!("getElementById"), 1, get_element_by_id);

        Ok(())
    }
}
