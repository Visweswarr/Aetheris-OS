// Minimal TS Intent Client helper
// Serializes an envelope and writes to INTENT_SOCKET; supports to-file:// fallback

import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import net from 'node:net';
import { encode, decode, writeFrame, readFrame } from './cbor_io.js';

export interface IntentEnvelopeV1 { v: number; captoken: Uint8Array; payload: Uint8Array; }
export interface IntentResponse { status: string; preview_id?: string; commit_id?: string; message?: string; }

export async function sendEnvelope(endpoint: string, env: IntentEnvelopeV1): Promise<IntentResponse> {
  if (!endpoint) throw new Error('INTENT_SOCKET not set');
  const enc = encode(env);
  if (endpoint.startsWith('to-file://')) {
    const p = endpoint.replace('to-file://', '');
    mkdirSync(dirname(p), { recursive: true });
    writeFileSync(p, Buffer.concat([enc, Buffer.from('\n')]), { flag: 'a' });
    return { status: 'ok', message: 'file' };
  }
  if (endpoint.startsWith('unix://') || endpoint.startsWith('pipe://')) {
    const path = endpoint.replace(/^unix:\/\//, '').replace(/^pipe:\/\//, '');
    const socket = net.createConnection(path);
    await new Promise((resolve, reject) => { socket.once('connect', resolve); socket.once('error', reject); });
    writeFrame(socket, enc);
    const respb = await readFrame(socket);
    socket.destroy();
    const resp = decode(respb) as IntentResponse;
    return resp;
  }
  throw new Error(`Unsupported endpoint: ${endpoint}`);
}
