import { Encoder, Decoder } from 'cbor-x';
import { Socket } from 'node:net';

const encoder = new Encoder({ useRecords: false, mapsAsObjects: true, structuredClone: true });
const decoder = new Decoder({ useRecords: false, mapsAsObjects: true });

export function encode(obj: any): Buffer {
  return Buffer.from(encoder.encode(obj));
}

export function decode(buf: Buffer): any {
  return decoder.decode(buf);
}

export function writeFrame(sock: Socket, buf: Buffer) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(buf.length, 0);
  sock.write(len);
  sock.write(buf);
}

export function readFrame(sock: Socket): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    let header = Buffer.alloc(0);
    let body: Buffer | null = null;
    let expected = 0;

    const onData = (chunk: Buffer) => {
      if (expected === 0) {
        header = Buffer.concat([header, chunk]);
        if (header.length >= 4) {
          expected = header.readUInt32BE(0);
          const rem = header.slice(4);
          body = rem.length ? Buffer.from(rem) : Buffer.alloc(0);
          if (body.length >= expected) {
            sock.off('data', onData);
            resolve(body.slice(0, expected));
          }
        }
      } else {
        body = Buffer.concat([body!, chunk]);
        if (body.length >= expected) {
          sock.off('data', onData);
          resolve(body.slice(0, expected));
        }
      }
    };

    sock.on('data', onData);
    sock.once('error', reject);
  });
}
