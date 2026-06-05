//! Node.js POSIX Networking Bridge
//! 
//! This module provides Node.js bindings for the Aetheris OS networking subsystem,
//! bridging Node.js net and dgram modules to the POSIX networking broker.

import { EventEmitter } from 'events';
import { Socket as NodeSocket } from 'net';
import { Socket as DgramSocket } from 'dgram';

// Network broker interface
interface NetworkBroker {
  createSocket(family: number, type: number, protocol: number): Promise<string>;
  bindSocket(socketId: string, address: string, port: number): Promise<void>;
  listenSocket(socketId: string, backlog: number): Promise<void>;
  acceptConnection(socketId: string): Promise<{ socketId: string; peerAddr: string }>;
  connectSocket(socketId: string, address: string, port: number): Promise<void>;
  sendData(socketId: string, data: Buffer, flags: number): Promise<number>;
  receiveData(socketId: string, bufferSize: number, flags: number): Promise<Buffer>;
  closeSocket(socketId: string): Promise<void>;
  setSocketOption(socketId: string, level: number, optname: number, value: Buffer): Promise<void>;
  getSocketOption(socketId: string, level: number, optname: number): Promise<Buffer>;
  subscribeReadiness(socketId: string, interests: number[], edgeTriggered: boolean): Promise<string>;
  unsubscribeReadiness(subscriptionId: string): Promise<void>;
  pollReadiness(socketIds: string[], interests: number[], timeout?: number): Promise<ReadinessEvent[]>;
}

// Readiness event
interface ReadinessEvent {
  socketId: string;
  interest: number;
  timestamp: number;
  data?: Buffer;
}

// Socket constants
const AF_INET = 2;
const AF_INET6 = 10;
const SOCK_STREAM = 1;
const SOCK_DGRAM = 2;
const SOL_SOCKET = 1;
const SO_NONBLOCK = 0x800;
const SO_REUSEADDR = 0x2;
const SO_KEEPALIVE = 0x9;
const SO_BROADCAST = 0x20;
const SO_RCVBUF = 0x1002;
const SO_SNDBUF = 0x1001;
const SO_ERROR = 0x1007;

// Readiness constants
const READINESS_READ = 0x001;
const READINESS_WRITE = 0x004;
const READINESS_ERROR = 0x008;
const READINESS_ACCEPT = 0x001;

// Mock broker implementation
class MockNetworkBroker implements NetworkBroker {
  private sockets: Map<string, SocketInfo> = new Map();
  private nextSocketId = 1;
  private nextFd = 3;

  async createSocket(family: number, type: number, protocol: number): Promise<string> {
    const socketId = `socket_${this.nextSocketId++}`;
    const fd = this.nextFd++;
    
    this.sockets.set(socketId, {
      id: socketId,
      fd,
      family,
      type,
      protocol,
      state: 'created',
      localAddr: null,
      peerAddr: null,
      nonBlocking: false,
      processCap: 'default',
      created: Date.now(),
    });

    console.log(`Broker: socket(family=${family}, type=${type}, protocol=${protocol}) -> fd=${fd}`);
    return socketId;
  }

  async bindSocket(socketId: string, address: string, port: number): Promise<void> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    socket.localAddr = { address, port };
    socket.state = 'bound';
    console.log(`Broker: bind(socketId=${socketId}, address=${address}, port=${port})`);
  }

  async listenSocket(socketId: string, backlog: number): Promise<void> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    if (socket.state !== 'bound') {
      throw new Error('Socket not bound');
    }

    socket.state = 'listening';
    console.log(`Broker: listen(socketId=${socketId}, backlog=${backlog})`);
  }

  async acceptConnection(socketId: string): Promise<{ socketId: string; peerAddr: string }> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    if (socket.state !== 'listening') {
      throw new Error('Socket not listening');
    }

    const newSocketId = `socket_${this.nextSocketId++}`;
    const newFd = this.nextFd++;
    const peerAddr = '127.0.0.1:12345';

    this.sockets.set(newSocketId, {
      id: newSocketId,
      fd: newFd,
      family: socket.family,
      type: socket.type,
      protocol: socket.protocol,
      state: 'connected',
      localAddr: socket.localAddr,
      peerAddr: { address: '127.0.0.1', port: 12345 },
      nonBlocking: socket.nonBlocking,
      processCap: socket.processCap,
      created: Date.now(),
    });

    console.log(`Broker: accept(socketId=${socketId}) -> newSocketId=${newSocketId}`);
    return { socketId: newSocketId, peerAddr };
  }

  async connectSocket(socketId: string, address: string, port: number): Promise<void> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    socket.peerAddr = { address, port };
    socket.state = 'connected';
    console.log(`Broker: connect(socketId=${socketId}, address=${address}, port=${port})`);
  }

  async sendData(socketId: string, data: Buffer, flags: number): Promise<number> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    if (socket.state !== 'connected') {
      throw new Error('Socket not connected');
    }

    console.log(`Broker: send(socketId=${socketId}, len=${data.length}, flags=${flags})`);
    return data.length;
  }

  async receiveData(socketId: string, bufferSize: number, flags: number): Promise<Buffer> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    if (socket.state !== 'connected') {
      throw new Error('Socket not connected');
    }

    console.log(`Broker: recv(socketId=${socketId}, bufferSize=${bufferSize}, flags=${flags})`);
    
    // Mock non-blocking behavior
    if (socket.nonBlocking) {
      throw new Error('Resource temporarily unavailable');
    }

    // Mock successful receive
    return Buffer.alloc(bufferSize);
  }

  async closeSocket(socketId: string): Promise<void> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    this.sockets.delete(socketId);
    console.log(`Broker: close(socketId=${socketId})`);
  }

  async setSocketOption(socketId: string, level: number, optname: number, value: Buffer): Promise<void> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    console.log(`Broker: setsockopt(socketId=${socketId}, level=${level}, optname=${optname}, value=${value})`);
    
    // Handle specific options
    if (level === SOL_SOCKET) {
      switch (optname) {
        case SO_NONBLOCK:
          socket.nonBlocking = value.length > 0 && value[0] !== 0;
          break;
        case SO_REUSEADDR:
        case SO_KEEPALIVE:
        case SO_BROADCAST:
          // Mock successful setting
          break;
        default:
          throw new Error('Socket option not supported');
      }
    }
  }

  async getSocketOption(socketId: string, level: number, optname: number): Promise<Buffer> {
    const socket = this.sockets.get(socketId);
    if (!socket) {
      throw new Error('Invalid socket');
    }

    console.log(`Broker: getsockopt(socketId=${socketId}, level=${level}, optname=${optname})`);
    
    // Mock socket option values
    switch (optname) {
      case SO_RCVBUF:
        return Buffer.alloc(4, 0);
      case SO_SNDBUF:
        return Buffer.alloc(4, 0);
      case SO_ERROR:
        return Buffer.alloc(4, 0);
      default:
        throw new Error('Socket option not supported');
    }
  }

  async subscribeReadiness(socketId: string, interests: number[], edgeTriggered: boolean): Promise<string> {
    const subscriptionId = `sub_${socketId}_${Date.now()}`;
    console.log(`Broker: subscribe_readiness(socketId=${socketId}, interests=${interests}, edgeTriggered=${edgeTriggered}) -> ${subscriptionId}`);
    return subscriptionId;
  }

  async unsubscribeReadiness(subscriptionId: string): Promise<void> {
    console.log(`Broker: unsubscribe_readiness(subscriptionId=${subscriptionId})`);
  }

  async pollReadiness(socketIds: string[], interests: number[], timeout?: number): Promise<ReadinessEvent[]> {
    console.log(`Broker: poll_readiness(socketIds=${socketIds}, interests=${interests}, timeout=${timeout})`);
    return [];
  }
}

// Socket information
interface SocketInfo {
  id: string;
  fd: number;
  family: number;
  type: number;
  protocol: number;
  state: string;
  localAddr: { address: string; port: number } | null;
  peerAddr: { address: string; port: number } | null;
  nonBlocking: boolean;
  processCap: string;
  created: number;
}

// Aetheris TCP Socket
export class AetherisTCPSocket extends EventEmitter {
  private broker: NetworkBroker;
  private socketId: string | null = null;
  private connected = false;
  private destroyed = false;

  constructor(broker?: NetworkBroker) {
    super();
    this.broker = broker || new MockNetworkBroker();
  }

  async connect(port: number, host?: string): Promise<void> {
    if (this.destroyed) {
      throw new Error('Socket is destroyed');
    }

    if (!this.socketId) {
      this.socketId = await this.broker.createSocket(AF_INET, SOCK_STREAM, 0);
    }

    const address = host || '127.0.0.1';
    await this.broker.connectSocket(this.socketId, address, port);
    this.connected = true;
    this.emit('connect');
  }

  async write(data: Buffer | string): Promise<boolean> {
    if (this.destroyed || !this.connected || !this.socketId) {
      return false;
    }

    const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
    const bytesSent = await this.broker.sendData(this.socketId, buffer, 0);
    this.emit('data', buffer);
    return bytesSent === buffer.length;
  }

  async read(size?: number): Promise<Buffer | null> {
    if (this.destroyed || !this.connected || !this.socketId) {
      return null;
    }

    try {
      const bufferSize = size || 4096;
      const data = await this.broker.receiveData(this.socketId, bufferSize, 0);
      return data;
    } catch (error) {
      if (error.message === 'Resource temporarily unavailable') {
        return null; // Non-blocking, no data available
      }
      throw error;
    }
  }

  async setNoDelay(noDelay?: boolean): Promise<void> {
    // Mock implementation
    console.log(`setNoDelay(${noDelay})`);
  }

  async setKeepAlive(enable?: boolean): Promise<void> {
    if (!this.socketId) return;
    
    const value = Buffer.alloc(4);
    value.writeInt32LE(enable ? 1 : 0, 0);
    await this.broker.setSocketOption(this.socketId, SOL_SOCKET, SO_KEEPALIVE, value);
  }

  async setNonBlocking(nonBlocking: boolean): Promise<void> {
    if (!this.socketId) return;
    
    const value = Buffer.alloc(4);
    value.writeInt32LE(nonBlocking ? 1 : 0, 0);
    await this.broker.setSocketOption(this.socketId, SOL_SOCKET, SO_NONBLOCK, value);
  }

  async destroy(): Promise<void> {
    if (this.destroyed || !this.socketId) {
      return;
    }

    this.destroyed = true;
    await this.broker.closeSocket(this.socketId);
    this.emit('close');
  }

  get connected(): boolean {
    return this.connected;
  }

  get destroyed(): boolean {
    return this.destroyed;
  }
}

// Aetheris UDP Socket
export class AetherisUDPSocket extends EventEmitter {
  private broker: NetworkBroker;
  private socketId: string | null = null;
  private bound = false;
  private destroyed = false;

  constructor(broker?: NetworkBroker) {
    super();
    this.broker = broker || new MockNetworkBroker();
  }

  async bind(port: number, address?: string): Promise<void> {
    if (this.destroyed) {
      throw new Error('Socket is destroyed');
    }

    if (!this.socketId) {
      this.socketId = await this.broker.createSocket(AF_INET, SOCK_DGRAM, 0);
    }

    const bindAddress = address || '0.0.0.0';
    await this.broker.bindSocket(this.socketId, bindAddress, port);
    this.bound = true;
    this.emit('listening');
  }

  async send(data: Buffer | string, port: number, address: string): Promise<number> {
    if (this.destroyed || !this.socketId) {
      throw new Error('Socket is destroyed');
    }

    const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
    const bytesSent = await this.broker.sendData(this.socketId, buffer, 0);
    return bytesSent;
  }

  async recv(size?: number): Promise<{ data: Buffer; address: string; port: number } | null> {
    if (this.destroyed || !this.socketId) {
      return null;
    }

    try {
      const bufferSize = size || 4096;
      const data = await this.broker.receiveData(this.socketId, bufferSize, 0);
      return {
        data,
        address: '127.0.0.1',
        port: 12345,
      };
    } catch (error) {
      if (error.message === 'Resource temporarily unavailable') {
        return null; // Non-blocking, no data available
      }
      throw error;
    }
  }

  async setBroadcast(flag: boolean): Promise<void> {
    if (!this.socketId) return;
    
    const value = Buffer.alloc(4);
    value.writeInt32LE(flag ? 1 : 0, 0);
    await this.broker.setSocketOption(this.socketId, SOL_SOCKET, SO_BROADCAST, value);
  }

  async setNonBlocking(nonBlocking: boolean): Promise<void> {
    if (!this.socketId) return;
    
    const value = Buffer.alloc(4);
    value.writeInt32LE(nonBlocking ? 1 : 0, 0);
    await this.broker.setSocketOption(this.socketId, SOL_SOCKET, SO_NONBLOCK, value);
  }

  async destroy(): Promise<void> {
    if (this.destroyed || !this.socketId) {
      return;
    }

    this.destroyed = true;
    await this.broker.closeSocket(this.socketId);
    this.emit('close');
  }

  get bound(): boolean {
    return this.bound;
  }

  get destroyed(): boolean {
    return this.destroyed;
  }
}

// Export broker for testing
export { NetworkBroker, MockNetworkBroker };