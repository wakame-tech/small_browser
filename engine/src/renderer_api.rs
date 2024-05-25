pub struct RendererAPI {}

impl RendererAPI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn rerender(&self) {
        // wasm側(renderer crate)に伝える必要があるが方法がわからないので後回しにする
        log::warn!("renderはtodo")
    }
}
