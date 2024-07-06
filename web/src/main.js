import init, { setup, render } from "./wasm";

const DEFAULT_HTML = `<body>
    <script>
    document.getElementById("result").innerText = "fuga";
    </script>
    <p>hello</p>
    <p class="inline">world</p>
    <p class="inline">:)</p>
    <p>this</p>
    <p class="inline">is</p>
    <p class="inline">inline</p>
    <div class="none">
        <p>this should not be shown</p>
    </div>
    <span id="result">hoge</span>
</body>
`;

const DEFAULT_CSS = `script {
    display: none;
}
p, div {
    display: block;
}
.none {
    display: none;
}
.inline {
    display: inline;
}`

const rerender = () => {
    const html = document.getElementById("html").value;
    const css = document.getElementById("css").value;
    render(html, css);
}

init().then(() => {
    setup();

    const html = document.getElementById("html")
    html.value = DEFAULT_HTML;
    const css = document.getElementById("css")
    css.value = DEFAULT_CSS;
    rerender();

    html.addEventListener("input", () => {
        rerender();
    })
    css.addEventListener("input", () => {
        rerender();
    })
});
