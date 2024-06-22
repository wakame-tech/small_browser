import init, { setup, render, exec_js } from "./wasm";

init().then(() => {
    setup();
    render();
    exec_js();
});
