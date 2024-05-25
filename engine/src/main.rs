use anyhow::Result;
use binding::create_document_object;
use dom::{dom::Node, html};
use renderer::Renderer;
use renderer_api::RendererAPI;
use std::{cell::RefCell, fs::File, io::Read, path::PathBuf, rc::Rc, sync::Once, time::Duration};

mod binding;
mod renderer;
mod renderer_api;

/// `JavaScriptRuntimeState` defines a state of JS runtime that will be stored per v8 isolate.
pub struct JavaScriptRuntimeState {
    pub context: v8::Global<v8::Context>,
    pub renderer_api: Rc<RendererAPI>,
    pub document_element: Rc<RefCell<Box<Node>>>,
}

/// `JavaScriptRuntime` defines a JS runtime with v8.
/// It has a link to a V8 isolate, and the isolate includes `JavaScriptRuntimeState` in its *slot*.
#[derive(Debug)]
pub struct JavaScriptRuntime {
    v8_isolate: v8::OwnedIsolate,
    document_element: Rc<RefCell<Box<Node>>>,
}

impl JavaScriptRuntime {
    pub fn new(document_element: Rc<RefCell<Box<Node>>>, renderer_api: Rc<RendererAPI>) -> Self {
        // init v8 platform just once
        static PUPPY_INIT: Once = Once::new();
        PUPPY_INIT.call_once(move || {
            let platform = v8::new_default_platform(1, false);
            v8::V8::initialize_platform(platform.into());
            v8::V8::initialize();
        });

        // create v8 isolate & context
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let context = {
            let isolate_scope = &mut v8::HandleScope::new(&mut isolate);
            let handle_scope = &mut v8::EscapableHandleScope::new(isolate_scope);
            let context = v8::Context::new(handle_scope);
            let global = context.global(handle_scope);
            {
                // `create_document_object()` の返り値をグローバル変数 `document` に格納する
                let scope = &mut v8::ContextScope::new(handle_scope, context);
                let key = v8::String::new(scope, "document").unwrap();
                let document = create_document_object(scope);
                global.set(scope, key.into(), document.into());
            }
            let context_scope = handle_scope.escape(context);
            v8::Global::new(handle_scope, context_scope)
        };

        // store state inside v8 isolate
        isolate.set_slot(Rc::new(RefCell::new(JavaScriptRuntimeState {
            context,
            renderer_api,
            document_element: document_element.clone(),
        })));

        JavaScriptRuntime {
            v8_isolate: isolate,
            document_element,
        }
    }

    /// `execute` runs a given source in the current context.
    pub fn execute(&mut self, filename: &str, source: &str) -> Result<String, String> {
        // `JavaScriptRuntimeState` から context handle scope を取り戻して開始
        let scope = &mut self.get_handle_scope();

        let source = v8::String::new(scope, source).unwrap();
        let source_map = v8::undefined(scope);
        let name = v8::String::new(scope, filename).unwrap();
        let origin = v8::ScriptOrigin::new(
            scope,
            name.into(),
            0,
            0,
            false,
            0,
            source_map.into(),
            false,
            false,
            false,
        );

        let mut tc_scope = v8::TryCatch::new(scope);
        let script = match v8::Script::compile(&mut tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                assert!(tc_scope.has_caught());
                return Err(to_pretty_string(tc_scope));
            }
        };

        match script.run(&mut tc_scope) {
            Some(result) => Ok(result
                .to_string(&mut tc_scope)
                .unwrap()
                .to_rust_string_lossy(&mut tc_scope)),
            None => {
                assert!(tc_scope.has_caught());
                Err(to_pretty_string(tc_scope))
            }
        }
    }
}

impl JavaScriptRuntime {
    /// `renderer_api` returns the `BrowserAPI` object in the Rust world linked to the given isolate.
    pub fn renderer_api(isolate: &v8::Isolate) -> Rc<RendererAPI> {
        let state = Self::state(isolate);
        let state = state.borrow();
        state.renderer_api.clone()
    }

    /// `get_renderer_api` returns the `BrowserAPI` object in the Rust world linked to the runtime.
    pub fn get_renderer_api(&mut self) -> Rc<RendererAPI> {
        Self::renderer_api(&self.v8_isolate)
    }

    /// `set_renderer_api` links the given `PageViewAPIHandler` object to the runtime.
    pub fn set_renderer_api(&mut self, renderer_api: Rc<RendererAPI>) {
        self.get_state().borrow_mut().renderer_api = renderer_api;
    }
}

impl JavaScriptRuntime {
    /// `document_element` returns the `Node` object in the Rust world linked to the given isolate.
    pub fn document_element(isolate: &v8::Isolate) -> Rc<RefCell<Box<Node>>> {
        let state = Self::state(isolate);
        let state = state.borrow();
        state.document_element.clone()
    }

    /// `get_document_element` returns the `Document` object in the Rust world linked to the runtime.
    pub fn get_document_element(&mut self) -> Rc<RefCell<Box<Node>>> {
        Self::document_element(&self.v8_isolate)
    }

    /// `set_document` links the given `Document` object to the runtime.
    pub fn set_document_element(&mut self, document_element: Rc<RefCell<Box<Node>>>) {
        self.get_state().borrow_mut().document_element = document_element;
    }
}

/// `JavaScriptRuntimeState` から状態を取り戻すための実装群
impl JavaScriptRuntime {
    /// `state` returns the runtime state stored in the given isolate.
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JavaScriptRuntimeState>> {
        let s = isolate
            .get_slot::<Rc<RefCell<JavaScriptRuntimeState>>>()
            .unwrap();
        s.clone()
    }

    /// `get_state` returns the runtime state for the runtime.
    pub fn get_state(&self) -> Rc<RefCell<JavaScriptRuntimeState>> {
        Self::state(&self.v8_isolate)
    }

    /// `get_handle_scope` returns [a handle scope](https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html) for the runtime.
    pub fn get_handle_scope(&mut self) -> v8::HandleScope {
        let context = self.get_context();
        v8::HandleScope::with_context(&mut self.v8_isolate, context)
    }

    /// `get_context` returns [a handle scope](https://v8docs.nodesource.com/node-0.8/df/d69/classv8_1_1_context.html) for the runtime.
    pub fn get_context(&mut self) -> v8::Global<v8::Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }
}

/// `to_pretty_string` formats the `TryCatch` instance into the prettified error string for puppy.
///
/// NOTE: See the following to get full error information.
/// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
fn to_pretty_string(mut try_catch: v8::TryCatch<v8::HandleScope>) -> String {
    // TODO (enhancement): better error handling needed! wanna remove uncareful unwrap().
    let exception_string = try_catch
        .exception()
        .unwrap()
        .to_string(&mut try_catch)
        .unwrap()
        .to_rust_string_lossy(&mut try_catch);
    let message = try_catch.message().unwrap();

    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );
    let line_number = message.get_line_number(&mut try_catch).unwrap_or_default();
    format!("{}:{}: {}", filename, line_number, exception_string)
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    setup_logger()?;
    let mut html_file = File::open("sample/sample.html")?;
    let mut html = String::new();
    html_file.read_to_string(&mut html)?;
    let node = html::parse(&html);
    node.write_as_bin(&PathBuf::from("sample/sample.html.bin"))?;
    std::thread::sleep(Duration::from_secs(3));
    let mut renderer = Renderer::new(node);
    renderer.execute_inline_scripts();

    Ok(())
}
