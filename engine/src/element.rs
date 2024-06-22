use boa_engine::{
    class::Class, js_string, property::Attribute, Context, Finalize, JsData, JsResult, JsValue,
    NativeFunction, Trace,
};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct Element;

impl Element {
    // TODO: innerHTML プロパティが読み出された際の処理
    fn get_inner_html(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // // `innerHTML` プロパティを持つオブジェクトを特定する
        // let this = args.this();

        // // Rust 上の DOM ツリーのノードを特定する
        // let node = to_linked_rust_node(scope, this);

        // // Rust 上の DOM ツリーのノードをテキスト表現に変換し、innerHTML プロパティの値として返す
        // let ret = v8::String::new(scope, node.inner_html().as_str()).unwrap();
        // rv.set(ret.into());

        Ok(JsValue::String(js_string!("get_inner_html")))
    }

    fn set_inner_html(
        _6his: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // // `innerHTML` プロパティを持つオブジェクトを特定する
        // let this = args.this();

        // // Rust 上の DOM ツリーのノードを特定する
        // let node = to_linked_rust_node(scope, this);

        // // Rust 上の DOM ツリー表現を更新する
        // let new_html = value.to_rust_string_lossy(scope);
        // log::debug!("new_html: {}", new_html);
        // node.set_inner_html(new_html.as_str());

        // JavaScriptRuntime::renderer_api(scope).rerender();
        Ok(JsValue::String(js_string!("set_inner_html")))
    }
}

impl Class for Element {
    const NAME: &'static str = "Element";

    fn data_constructor(
        new_target: &boa_engine::JsValue,
        args: &[boa_engine::JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(Element)
    }

    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        // create `tagName` property
        class.property(
            js_string!("tagName"),
            JsValue::String(js_string!("p")),
            Attribute::READONLY,
        );

        // create `innerHTML` property
        let get_inner_html = NativeFunction::from_fn_ptr(Self::get_inner_html)
            .to_js_function(class.context().realm());
        let set_inner_html = NativeFunction::from_fn_ptr(Self::set_inner_html)
            .to_js_function(class.context().realm());
        class.accessor(
            js_string!("innerHTML"),
            Some(get_inner_html),
            Some(set_inner_html),
            Attribute::all(),
        );

        Ok(())
    }
}
