import init, { render } from "./wasm";

init().then(() => {
    setInterval(async () => {
        const buf = await fetch('http://localhost:8000').then(res => res.arrayBuffer());
        render(buf)
    }, 1000)
});
