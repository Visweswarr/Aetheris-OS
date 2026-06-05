import { sendEnvelope } from "./intent_client.js";
import { encode as cborEncode } from "./cbor_io.js";
import * as fs from "node:fs";

(async () => {
  const endpoint = process.env.INTENT_SOCKET || "";
  const tokPath = process.env.AICORE_CAPTOKEN || "";
  if (!endpoint || !tokPath) { console.error("missing env"); process.exit(4); }
  const captoken = fs.readFileSync(tokPath);
  const env = { v: 1, captoken: new Uint8Array(captoken), payload: new Uint8Array(cborEncode({})) } as const;
  try {
    const resp = await sendEnvelope(endpoint, env as any);
    if (resp?.status === "ok") { console.log("ok"); process.exit(0); }
    if (resp?.status === "deny") { console.log("deny"); process.exit(2); }
    console.error("unexpected:", resp); process.exit(3);
  } catch (e) {
    console.error("error:", e); process.exit(3);
  }
})();
