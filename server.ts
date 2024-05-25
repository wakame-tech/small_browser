import { Application, Router } from "https://deno.land/x/oak/mod.ts";
import { oakCors } from "https://deno.land/x/cors/mod.ts";

const router = new Router();
router
    .get("/", async (context) => {
        const bin = await Deno.readFile("sample/sample.html.bin");
        console.log(`GET / (${bin.length} bytes)`);
        context.response.body = bin;
    })

const app = new Application();
app.use(oakCors()); // Enable CORS for All Routes
app.use(router.routes());

console.info("CORS-enabled web server listening on port 8000");
await app.listen({ port: 8000 });
