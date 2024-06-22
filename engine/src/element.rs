use crate::DOM;
use boa_engine::{
    class::{Class, ClassBuilder},
    js_string,
    property::Attribute,
    Context, Finalize, JsData, JsError, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    Trace,
};
use dom::dom::NodeType;

#[derive(Debug, Trace, Finalize, JsData)]
pub struct Element {
    pub id: String,
}

impl Element {
    fn get_id(this: &JsValue) -> JsResult<String> {
        let this = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("get Element.tagName called with invalid value")
            })?;
        Ok(this.id.clone())
    }

    fn get_tag_name(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let id = Self::get_id(this)?;
        let mut dom = DOM.try_lock().unwrap();
        let Some(node) = dom.get_element_by_id(&id) else {
            return Err(JsError::from_native(
                JsNativeError::typ().with_message(format!("{} not found", id)),
            ));
        };
        let NodeType::Element(element) = &node.node_type else {
            return Err(JsError::from_native(
                JsNativeError::typ().with_message(format!("{} is not an element", id)),
            ));
        };
        Ok(JsValue::String(js_string!(element.tag_name.clone())))
    }

    fn get_inner_text(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let id = Self::get_id(this)?;
        let mut dom = DOM.try_lock().unwrap();
        let Some(node) = dom.get_element_by_id(&id) else {
            return Err(JsError::from_native(
                JsNativeError::typ().with_message(format!("{} not found", id)),
            ));
        };
        Ok(JsValue::String(js_string!(node.inner_text())))
    }

    fn set_inner_text(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let text = args[0].as_string().unwrap().to_std_string_escaped();
        let id = Self::get_id(this)?;
        let mut dom = DOM.try_lock().unwrap();
        let Some(node) = dom.get_element_by_id(&id) else {
            return Err(JsError::from_native(
                JsNativeError::typ().with_message(format!("{} not found", id)),
            ));
        };
        node.set_inner_text(&text);
        Ok(JsValue::String(js_string!(text)))
    }
}

impl Class for Element {
    const NAME: &'static str = "Element";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(Element { id: "".to_string() })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        // create `tagName` property
        let get_tag_name =
            NativeFunction::from_fn_ptr(Self::get_tag_name).to_js_function(class.context().realm());
        class.accessor(
            js_string!("tagName"),
            Some(get_tag_name),
            None,
            Attribute::READONLY,
        );

        // create `innerText` property
        let get_inner_text = NativeFunction::from_fn_ptr(Self::get_inner_text)
            .to_js_function(class.context().realm());
        let set_inner_text = NativeFunction::from_fn_ptr(Self::set_inner_text)
            .to_js_function(class.context().realm());
        class.accessor(
            js_string!("innerText"),
            Some(get_inner_text),
            Some(set_inner_text),
            Attribute::all(),
        );

        Ok(())
    }
}
