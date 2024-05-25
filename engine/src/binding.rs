use crate::JavaScriptRuntime;
use dom::dom::{Node, NodeType};
use std::ffi::c_void;

type NodeRefTarget<'a> = &'a mut Box<Node>;

/// `to_v8_node` returns the v8 representation of the given `Node` object.
fn to_v8_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    // create new node instance
    let template = v8::ObjectTemplate::new(scope);
    template.set_internal_field_count(1);
    let node_v8 = template.new_instance(scope).unwrap();

    // set a reference to Node into the internal field
    let boxed_ref = Box::new(node_rust);
    let addr = Box::leak(boxed_ref) as *mut NodeRefTarget as *mut c_void;
    let v8_ext = v8::External::new(scope, addr);
    let target_node_ref_v8: v8::Local<v8::Value> = v8_ext.into();
    node_v8.set_internal_field(0, target_node_ref_v8.into());

    // all set :-)
    node_v8
}

/// `to_linked_rust_node` returns a `Node` object that corresponds with the given `node_v8`.
fn to_linked_rust_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_v8: v8::Local<v8::Object>,
) -> &'s mut NodeRefTarget<'s> {
    let node_v8 = node_v8.get_internal_field(scope, 0).unwrap();
    let node = unsafe { v8::Local::<v8::External>::cast(node_v8) };
    let node = node.value() as *mut NodeRefTarget;
    unsafe { &mut *node }
}

fn to_v8_element<'s>(
    scope: &mut v8::HandleScope<'s>,
    tag_name: &str,
    _attributes: Vec<(String, String)>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    let node = to_v8_node(scope, node_rust);

    // create properties of the node
    {
        // create `tagName` property
        let key = v8::String::new(scope, "tagName").unwrap();
        let value = v8::String::new(scope, tag_name).unwrap();
        node.define_own_property(
            scope,
            key.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY,
        );
    }

    {
        // create `innerHTML` property
        let key = v8::String::new(scope, "innerHTML").unwrap();
        node.set_accessor_with_setter(
            scope,
            key.into(),
            // innerHTML プロパティが読み出された際の処理
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  args: v8::PropertyCallbackArguments,
                  mut rv: v8::ReturnValue| {
                // `innerHTML` プロパティを持つオブジェクトを特定する
                let this = args.this();

                // Rust 上の DOM ツリーのノードを特定する
                let node = to_linked_rust_node(scope, this);

                // Rust 上の DOM ツリーのノードをテキスト表現に変換し、innerHTML プロパティの値として返す
                let ret = v8::String::new(scope, node.inner_html().as_str()).unwrap();
                rv.set(ret.into());
            },
            // innerHTML プロパティへの代入があった際の処理
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  value: v8::Local<v8::Value>,
                  args: v8::PropertyCallbackArguments,
                  _: v8::ReturnValue| {
                // `innerHTML` プロパティを持つオブジェクトを特定する
                let this = args.this();

                // Rust 上の DOM ツリーのノードを特定する
                let node = to_linked_rust_node(scope, this);

                // Rust 上の DOM ツリー表現を更新する
                let new_html = value.to_rust_string_lossy(scope);
                log::debug!("new_html: {}", new_html);
                node.set_inner_html(new_html.as_str());

                JavaScriptRuntime::renderer_api(scope).rerender();
            },
        );
    }

    node
}

pub fn create_document_object<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
) -> v8::Local<'s, v8::Object> {
    let document = v8::ObjectTemplate::new(scope).new_instance(scope).unwrap();

    {
        // `getElementById()` 関数の定義
        let key = v8::String::new(scope, "getElementById").unwrap();
        let tmpl = v8::FunctionTemplate::new(
            scope,
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut retval: v8::ReturnValue| {
                // `getElementById()` 関数の引数を取り出す
                let id = args
                    .get(0)
                    .to_string(scope)
                    .unwrap()
                    .to_rust_string_lossy(scope);

                // 現在描画されている DOM ツリーの Rust 上での表現を特定する
                let document_element = JavaScriptRuntime::document_element(scope);
                let document_element = &mut document_element.borrow_mut();

                // `getElementById()` の引数を用いて DOM ツリーを検索し、返り値を設定する
                retval.set(
                    document_element
                        .get_element_by_id(id.as_str())
                        .and_then(|n| {
                            if let NodeType::Element(ref mut e) = n.node_type {
                                let tag_name = e.tag_name.clone();
                                let attributes = e.attributes();
                                Some((n, tag_name, attributes))
                            } else {
                                None
                            }
                        })
                        .and_then(|(n, tag_name, attributes)| {
                            Some(to_v8_element(scope, tag_name.as_str(), attributes, n).into())
                        })
                        .unwrap_or_else(|| v8::undefined(scope).into()),
                );
            },
        );
        let val = tmpl.get_function(scope).unwrap();
        document.set(scope, key.into(), val.into());
    }

    document
}