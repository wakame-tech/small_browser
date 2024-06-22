use crate::{element::Element, DOM};
use boa_engine::{
    class::{Class, ClassBuilder},
    js_string, Context, Finalize, JsData, JsError, JsResult, JsValue, NativeFunction, Trace,
};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct Document;

impl Document {
    fn get_element_by_id(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let id = args[0].as_string().unwrap().to_std_string_escaped();
        let mut dom = DOM.try_lock().unwrap();
        if dom.get_element_by_id(id.as_str()).is_none() {
            return Err(JsError::from_opaque(JsValue::String(js_string!(format!(
                "get_element_by_id #{} not found",
                id
            )))));
        };
        let element = Element::from_data(Element { id }, context).unwrap();
        Ok(JsValue::Object(element))
    }
}

impl Class for Document {
    const NAME: &'static str = "Document";

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(Document)
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        // `getElementById()` 関数の定義
        let get_element_by_id = NativeFunction::from_fn_ptr(Self::get_element_by_id);
        class.method(js_string!("getElementById"), 1, get_element_by_id);

        Ok(())
    }
}
