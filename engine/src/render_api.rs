pub struct RenderAPI {}

impl RenderAPI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self) {
        // wasm側(renderer crate)に伝える必要があるが方法がわからないので後回しにする
        log::warn!("renderはtodo")
    }
}
