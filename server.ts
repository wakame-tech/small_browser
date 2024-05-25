import { Application, Router, send } from "https://deno.land/x/oak/mod.ts";
import { oakCors } from "https://deno.land/x/cors/mod.ts";

const bin = await Deno.readFile("sample/sample.html.bin");
console.log(bin)

const router = new Router();
router
    .get("/", async (context) => {
        context.response.body = bin;
    })

const app = new Application();
app.use(oakCors()); // Enable CORS for All Routes
app.use(router.routes());

console.info("CORS-enabled web server listening on port 8000");
await app.listen({ port: 8000 });
