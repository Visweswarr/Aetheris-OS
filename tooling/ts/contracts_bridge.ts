/**
 * TypeScript SDK for Aetheris OS Smart Contracts
 * 
 * This module provides client-side interaction with the Aetheris OS smart contract system,
 * including deployment, execution, querying, and resource management.
 */

import { EventEmitter } from 'events';

// Type definitions
export interface ContractMetadata {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  wasmHash: string;
  createdAt: number;
  updatedAt: number;
}

export interface ExecutionResult {
  success: boolean;
  output: Uint8Array;
  gasUsed: number;
  executionTime: number;
  events: ContractEvent[];
  error?: string;
}

export interface ContractEvent {
  eventType: string;
  data: Record<string, string>;
  timestamp: number;
}

export interface ExecutionRequest {
  contractId: string;
  method: string;
  args: Uint8Array;
  gasLimit: number;
  caller: string;
  value: number;
}

export interface DeploymentRequest {
  wasm: Uint8Array;
  metadata: ContractMetadata;
  gasLimit: number;
  deployer: string;
  value: number;
}

export interface ResourceAllocation {
  contractId: string;
  cpuLimit: number;
  memoryLimit: number;
  storageLimit: number;
  networkLimit: number;
  allocatedBy: string;
  allocatedAt: number;
}

export interface ContractState {
  contractId: string;
  storage: Record<string, Uint8Array>;
  balance: number;
  lastExecution?: number;
  executionCount: number;
}

export interface ContractConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enablePQC: boolean;
  enableZKProofs: boolean;
}

export interface ChainConfig {
  chainType: 'ethereum' | 'polkadot' | 'solana' | 'avalanche' | 'polygon' | 'arbitrum' | 'optimism';
  rpcUrl: string;
  chainId: number;
  gasPrice: number;
  gasLimit: number;
  timeout: number;
  retryCount: number;
  enablePQC: boolean;
}

export interface TransactionRequest {
  id: string;
  chainType: string;
  to: string;
  data: Uint8Array;
  value: number;
  gasLimit: number;
  gasPrice: number;
  nonce?: number;
  signature?: Uint8Array;
  createdAt: number;
}

export interface TransactionResult {
  id: string;
  txHash: string;
  status: 'pending' | 'confirmed' | 'failed' | 'reverted';
  gasUsed: number;
  blockNumber?: number;
  error?: string;
  executedAt: number;
}

export interface RelayStats {
  totalTransactions: number;
  successfulTransactions: number;
  failedTransactions: number;
  averageGasUsed: number;
  averageExecutionTime: number;
  chainStats: Record<string, ChainStats>;
}

export interface ChainStats {
  chainType: string;
  totalTransactions: number;
  successfulTransactions: number;
  failedTransactions: number;
  averageGasUsed: number;
  averageExecutionTime: number;
  lastBlockNumber: number;
  isConnected: boolean;
}

/**
 * Main Contracts Bridge class
 */
export class ContractsBridge extends EventEmitter {
  private config: ContractConfig;
  private chainConfigs: Map<string, ChainConfig>;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;

  constructor(config: Partial<ContractConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enablePQC: config.enablePQC || true,
      enableZKProofs: config.enableZKProofs || true,
    };

    this.chainConfigs = new Map();
  }

  /**
   * Initialize the bridge connection
   */
  async initialize(): Promise<void> {
    try {
      await this.connect();
      this.isConnected = true;
      this.connectionRetries = 0;
      this.emit('connected');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Connect to the contract service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to contract service: ${response.statusText}`);
    }
  }

  /**
   * Deploy a WASM smart contract
   */
  async deployContract(request: DeploymentRequest): Promise<{ contractId: string; gasUsed: number; deploymentTime: number }> {
    await this.ensureConnected();

    const response = await this.makeRequest('/contracts/deploy', {
      method: 'POST',
      body: JSON.stringify({
        wasm: Array.from(request.wasm),
        metadata: request.metadata,
        gas_limit: request.gasLimit,
        deployer: request.deployer,
        value: request.value,
      }),
    });

    if (!response.success) {
      throw new Error(`Contract deployment failed: ${response.error}`);
    }

    this.emit('contractDeployed', {
      contractId: response.contract_id,
      metadata: request.metadata,
    });

    return {
      contractId: response.contract_id,
      gasUsed: response.gas_used,
      deploymentTime: response.deployment_time,
    };
  }

  /**
   * Call a smart contract method
   */
  async callContract(request: ExecutionRequest): Promise<ExecutionResult> {
    await this.ensureConnected();

    const response = await this.makeRequest('/contracts/call', {
      method: 'POST',
      body: JSON.stringify({
        contract_id: request.contractId,
        method: request.method,
        args: Array.from(request.args),
        gas_limit: request.gasLimit,
        caller: request.caller,
        value: request.value,
      }),
    });

    if (!response.success) {
      throw new Error(`Contract call failed: ${response.error}`);
    }

    const result: ExecutionResult = {
      success: response.success,
      output: new Uint8Array(response.output),
      gasUsed: response.gas_used,
      executionTime: response.execution_time,
      events: response.events || [],
      error: response.error,
    };

    this.emit('contractCalled', {
      contractId: request.contractId,
      method: request.method,
      result,
    });

    return result;
  }

  /**
   * Query a smart contract state
   */
  async queryContract(contractId: string, method: string, args: Uint8Array = new Uint8Array()): Promise<ExecutionResult> {
    await this.ensureConnected();

    const response = await this.makeRequest('/contracts/query', {
      method: 'POST',
      body: JSON.stringify({
        contract_id: contractId,
        method,
        args: Array.from(args),
      }),
    });

    if (!response.success) {
      throw new Error(`Contract query failed: ${response.error}`);
    }

    return {
      success: response.success,
      output: new Uint8Array(response.output),
      gasUsed: 0,
      executionTime: response.execution_time,
      events: [],
      error: response.error,
    };
  }

  /**
   * List all deployed contracts
   */
  async listContracts(): Promise<ContractMetadata[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/contracts/list', {
      method: 'GET',
    });

    return response.contracts || [];
  }

  /**
   * Get contract status and information
   */
  async getContractStatus(contractId: string): Promise<{
    metadata: ContractMetadata;
    state: ContractState;
    allocation?: ResourceAllocation;
  }> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/contracts/${contractId}/status`, {
      method: 'GET',
    });

    return {
      metadata: response.metadata,
      state: response.state,
      allocation: response.allocation,
    };
  }

  /**
   * Allocate resources to a contract
   */
  async allocateResources(contractId: string, allocation: Omit<ResourceAllocation, 'contractId' | 'allocatedBy' | 'allocatedAt'>): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/contracts/${contractId}/allocate`, {
      method: 'POST',
      body: JSON.stringify({
        cpu_limit: allocation.cpuLimit,
        memory_limit: allocation.memoryLimit,
        storage_limit: allocation.storageLimit,
        network_limit: allocation.networkLimit,
      }),
    });

    if (!response.success) {
      throw new Error(`Resource allocation failed: ${response.error}`);
    }

    this.emit('resourcesAllocated', {
      contractId,
      allocation,
    });
  }

  /**
   * Submit a transaction to the relay
   */
  async submitTransaction(request: TransactionRequest): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/relay/submit', {
      method: 'POST',
      body: JSON.stringify({
        id: request.id,
        chain_type: request.chainType,
        to: request.to,
        data: Array.from(request.data),
        value: request.value,
        gas_limit: request.gasLimit,
        gas_price: request.gasPrice,
        nonce: request.nonce,
        signature: request.signature ? Array.from(request.signature) : undefined,
        created_at: request.createdAt,
      }),
    });

    if (!response.success) {
      throw new Error(`Transaction submission failed: ${response.error}`);
    }

    this.emit('transactionSubmitted', {
      transactionId: request.id,
      chainType: request.chainType,
    });

    return response.transaction_id;
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(transactionId: string): Promise<TransactionResult | null> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/relay/transactions/${transactionId}`, {
      method: 'GET',
    });

    if (!response.success) {
      return null;
    }

    return {
      id: response.id,
      txHash: response.tx_hash,
      status: response.status,
      gasUsed: response.gas_used,
      blockNumber: response.block_number,
      error: response.error,
      executedAt: response.executed_at,
    };
  }

  /**
   * Get relay statistics
   */
  async getRelayStats(): Promise<RelayStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/relay/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get chain statistics
   */
  async getChainStats(chainType: string): Promise<ChainStats | null> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/relay/chains/${chainType}/stats`, {
      method: 'GET',
    });

    if (!response.success) {
      return null;
    }

    return response;
  }

  /**
   * Check chain connectivity
   */
  async checkChainConnectivity(chainType: string): Promise<boolean> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/relay/chains/${chainType}/connectivity`, {
      method: 'GET',
    });

    return response.connected || false;
  }

  /**
   * Get supported chains
   */
  async getSupportedChains(): Promise<string[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/relay/chains', {
      method: 'GET',
    });

    return response.chains || [];
  }

  /**
   * Add chain configuration
   */
  addChainConfig(chainType: string, config: ChainConfig): void {
    this.chainConfigs.set(chainType, config);
  }

  /**
   * Remove chain configuration
   */
  removeChainConfig(chainType: string): void {
    this.chainConfigs.delete(chainType);
  }

  /**
   * Get chain configuration
   */
  getChainConfig(chainType: string): ChainConfig | undefined {
    return this.chainConfigs.get(chainType);
  }

  /**
   * Get all chain configurations
   */
  getAllChainConfigs(): Map<string, ChainConfig> {
    return new Map(this.chainConfigs);
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<ContractConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): ContractConfig {
    return { ...this.config };
  }

  /**
   * Check if connected
   */
  isBridgeConnected(): boolean {
    return this.isConnected;
  }

  /**
   * Disconnect from the service
   */
  async disconnect(): Promise<void> {
    this.isConnected = false;
    this.emit('disconnected');
  }

  /**
   * Ensure connection is established
   */
  private async ensureConnected(): Promise<void> {
    if (!this.isConnected) {
      await this.initialize();
    }
  }

  /**
   * Make HTTP request with retry logic
   */
  private async makeRequest(endpoint: string, options: RequestInit): Promise<any> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= this.config.retries; attempt++) {
      try {
        const response = await fetch(`${this.config.endpoint}${endpoint}`, {
          ...options,
          headers: {
            'Content-Type': 'application/json',
            ...options.headers,
          },
          signal: AbortSignal.timeout(this.config.timeout),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        return data;
      } catch (error) {
        lastError = error as Error;
        
        if (attempt < this.config.retries) {
          const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }

    throw lastError || new Error('Request failed after all retries');
  }
}

/**
 * Utility functions for working with contracts
 */
export class ContractUtils {
  /**
   * Generate a unique contract ID
   */
  static generateContractId(): string {
    return `contract_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate a unique transaction ID
   */
  static generateTransactionId(): string {
    return `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Convert string to Uint8Array
   */
  static stringToUint8Array(str: string): Uint8Array {
    return new TextEncoder().encode(str);
  }

  /**
   * Convert Uint8Array to string
   */
  static uint8ArrayToString(data: Uint8Array): string {
    return new TextDecoder().decode(data);
  }

  /**
   * Convert object to Uint8Array (JSON serialization)
   */
  static objectToUint8Array(obj: any): Uint8Array {
    return this.stringToUint8Array(JSON.stringify(obj));
  }

  /**
   * Convert Uint8Array to object (JSON deserialization)
   */
  static uint8ArrayToObject(data: Uint8Array): any {
    return JSON.parse(this.uint8ArrayToString(data));
  }

  /**
   * Calculate gas estimate for a contract call
   */
  static estimateGas(contractSize: number, methodComplexity: number, dataSize: number): number {
    const baseGas = 21000;
    const contractGas = contractSize * 10;
    const methodGas = methodComplexity * 1000;
    const dataGas = dataSize * 16;
    
    return baseGas + contractGas + methodGas + dataGas;
  }

  /**
   * Validate contract metadata
   */
  static validateMetadata(metadata: Partial<ContractMetadata>): string[] {
    const errors: string[] = [];

    if (!metadata.name || metadata.name.trim().length === 0) {
      errors.push('Contract name is required');
    }

    if (!metadata.version || metadata.version.trim().length === 0) {
      errors.push('Contract version is required');
    }

    if (!metadata.author || metadata.author.trim().length === 0) {
      errors.push('Contract author is required');
    }

    if (metadata.name && metadata.name.length > 100) {
      errors.push('Contract name must be less than 100 characters');
    }

    if (metadata.description && metadata.description.length > 1000) {
      errors.push('Contract description must be less than 1000 characters');
    }

    return errors;
  }

  /**
   * Create default contract metadata
   */
  static createDefaultMetadata(name: string, author: string, description?: string): ContractMetadata {
    return {
      id: this.generateContractId(),
      name,
      version: '1.0.0',
      author,
      description: description || '',
      wasmHash: '',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
  }
}

/**
 * Event types for the ContractsBridge
 */
export interface ContractsBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  contractDeployed: (data: { contractId: string; metadata: ContractMetadata }) => void;
  contractCalled: (data: { contractId: string; method: string; result: ExecutionResult }) => void;
  resourcesAllocated: (data: { contractId: string; allocation: ResourceAllocation }) => void;
  transactionSubmitted: (data: { transactionId: string; chainType: string }) => void;
}

/**
 * Default export
 */
export default ContractsBridge;
