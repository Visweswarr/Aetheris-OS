import React, { useState, useEffect, useCallback } from 'react';
import './contract_ui.css';

// Contract types matching the CDDL schema
interface ContractV1 {
  id: string;
  wasm: Uint8Array;
  meta: ContractMeta;
  zk_mode: boolean;
  gas_limit: number;
  version: string;
  created_at: string;
  updated_at: string;
  owner_did: string;
  capabilities: string[];
  tags: string[];
  dependencies: string[];
}

interface ContractMeta {
  name: string;
  description?: string;
  author: string;
  license?: string;
  category: ContractCategory;
  interfaces: string[];
  storage_keys: string[];
  schemas: Record<string, any>;
  gas_estimate?: number;
  memory?: number;
  timeout?: number;
}

enum ContractCategory {
  UTILITY = 'utility',
  FINANCIAL = 'financial',
  GOVERNANCE = 'governance',
  IDENTITY = 'identity',
  STORAGE = 'storage',
  COMPUTATION = 'computation',
  CUSTOM = 'custom'
}

interface ResultV1 {
  ok: boolean;
  gas_used: number;
  output: Uint8Array;
  proof?: ZKProof;
  error?: ContractError;
  time: number;
  memory: number;
  storage_accesses: StorageAccess[];
  events: ContractEvent[];
  timestamp: string;
  contract_id: string;
  input_hash: string;
}

interface ZKProof {
  algorithm: ZKAlgorithm;
  proof_data: Uint8Array;
  public_inputs: Uint8Array[];
  circuit_hash: Uint8Array;
  prover_version: string;
  proof_size: number;
  verification_key: Uint8Array;
}

enum ZKAlgorithm {
  HALO2 = 'halo2',
  NOIR = 'noir',
  PLONK = 'plonk',
  CUSTOM = 'custom'
}

interface ContractError {
  code: ErrorCode;
  message: string;
  details?: string;
  gas_consumed: number;
  instruction_count: number;
  memory_peak: number;
}

enum ErrorCode {
  GAS_LIMIT_EXCEEDED = 'gas_limit_exceeded',
  MEMORY_LIMIT_EXCEEDED = 'memory_limit_exceeded',
  TIMEOUT = 'timeout',
  INVALID_INPUT = 'invalid_input',
  RUNTIME_ERROR = 'runtime_error',
  ZK_PROOF_FAILED = 'zk_proof_failed',
  INSUFFICIENT_CAPABILITIES = 'insufficient_capabilities'
}

interface StorageAccess {
  key: string;
  operation: StorageOp;
  value?: Uint8Array;
  timestamp: string;
}

enum StorageOp {
  READ = 'read',
  WRITE = 'write',
  DELETE = 'delete'
}

interface ContractEvent {
  name: string;
  data: Record<string, any>;
  timestamp: string;
  block_number?: number;
}

// Execution options
interface ExecutionOptions {
  gas_limit: number;
  zk_mode: boolean;
  algorithm: ZKAlgorithm;
  timeout_ms: number;
  input_data: string;
}

// Contract execution state
interface ExecutionState {
  isExecuting: boolean;
  result?: ResultV1;
  error?: string;
  executionTime: number;
}

// Main contract explorer component
export const ContractExplorer: React.FC = () => {
  const [contracts, setContracts] = useState<ContractV1[]>([]);
  const [selectedContract, setSelectedContract] = useState<ContractV1 | null>(null);
  const [executionState, setExecutionState] = useState<ExecutionState>({
    isExecuting: false,
    executionTime: 0
  });
  const [executionOptions, setExecutionOptions] = useState<ExecutionOptions>({
    gas_limit: 1000000,
    zk_mode: false,
    algorithm: ZKAlgorithm.HALO2,
    timeout_ms: 30000,
    input_data: '{"value": 42}'
  });
  const [searchTerm, setSearchTerm] = useState('');
  const [filterCategory, setFilterCategory] = useState<ContractCategory | 'all'>('all');
  const [sortBy, setSortBy] = useState<'name' | 'category' | 'created_at' | 'gas_limit'>('name');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

  // Load mock contracts on component mount
  useEffect(() => {
    loadMockContracts();
  }, []);

  const loadMockContracts = useCallback(() => {
    const mockContracts: ContractV1[] = [
      {
        id: 'increment-contract',
        wasm: new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]),
        meta: {
          name: 'Increment Contract',
          description: 'Simple counter increment with deterministic output',
          author: 'NGFS Team',
          license: 'MIT',
          category: ContractCategory.UTILITY,
          interfaces: ['increment', 'get_value'],
          storage_keys: ['counter'],
          schemas: {},
          gas_estimate: 1000,
          memory: 1024,
          timeout: 5000
        },
        zk_mode: true,
        gas_limit: 1000000,
        version: '1.0.0',
        created_at: '2024-01-15T10:00:00Z',
        updated_at: '2024-01-15T10:00:00Z',
        owner_did: 'did:aetheris:team:ngfs',
        capabilities: ['execute', 'read'],
        tags: ['counter', 'simple', 'demo'],
        dependencies: []
      },
      {
        id: 'kv-store-contract',
        wasm: new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]),
        meta: {
          name: 'Key-Value Store',
          description: 'Persistent key-value storage with encryption',
          author: 'NGFS Team',
          license: 'MIT',
          category: ContractCategory.STORAGE,
          interfaces: ['set', 'get', 'delete', 'list'],
          storage_keys: ['data', 'metadata'],
          schemas: {
            key: 'string',
            value: 'bytes',
            metadata: 'object'
          },
          gas_estimate: 5000,
          memory: 8192,
          timeout: 10000
        },
        zk_mode: true,
        gas_limit: 5000000,
        version: '1.0.0',
        created_at: '2024-01-16T14:30:00Z',
        updated_at: '2024-01-16T14:30:00Z',
        owner_did: 'did:aetheris:team:ngfs',
        capabilities: ['execute', 'read', 'write'],
        tags: ['storage', 'kv', 'encrypted'],
        dependencies: []
      },
      {
        id: 'validator-contract',
        wasm: new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]),
        meta: {
          name: 'Data Validator',
          description: 'Validates data structures and generates ZK proofs',
          author: 'NGFS Team',
          license: 'MIT',
          category: ContractCategory.COMPUTATION,
          interfaces: ['validate', 'generate_proof', 'verify_proof'],
          storage_keys: ['validation_rules', 'proof_cache'],
          schemas: {
            data: 'any',
            rules: 'array',
            proof: 'object'
          },
          gas_estimate: 8000,
          memory: 16384,
          timeout: 15000
        },
        zk_mode: true,
        gas_limit: 10000000,
        version: '1.0.0',
        created_at: '2024-01-17T09:15:00Z',
        updated_at: '2024-01-17T09:15:00Z',
        owner_did: 'did:aetheris:team:ngfs',
        capabilities: ['execute', 'read', 'write', 'validate'],
        tags: ['validation', 'zk', 'proofs'],
        dependencies: []
      }
    ];

    setContracts(mockContracts);
  }, []);

  // Filter and sort contracts
  const filteredAndSortedContracts = useCallback(() => {
    let filtered = contracts.filter(contract => {
      const matchesSearch = contract.meta.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           contract.meta.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           contract.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
      
      const matchesCategory = filterCategory === 'all' || contract.meta.category === filterCategory;
      
      return matchesSearch && matchesCategory;
    });

    // Sort contracts
    filtered.sort((a, b) => {
      let aValue: any, bValue: any;
      
      switch (sortBy) {
        case 'name':
          aValue = a.meta.name;
          bValue = b.meta.name;
          break;
        case 'category':
          aValue = a.meta.category;
          bValue = b.meta.category;
          break;
        case 'created_at':
          aValue = new Date(a.created_at);
          bValue = new Date(b.created_at);
          break;
        case 'gas_limit':
          aValue = a.gas_limit;
          bValue = b.gas_limit;
          break;
        default:
          aValue = a.meta.name;
          bValue = b.meta.name;
      }

      if (sortOrder === 'asc') {
        return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
      } else {
        return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
      }
    });

    return filtered;
  }, [contracts, searchTerm, filterCategory, sortBy, sortOrder]);

  // Execute contract
  const executeContract = useCallback(async () => {
    if (!selectedContract) return;

    setExecutionState(prev => ({ ...prev, isExecuting: true, error: undefined, result: undefined }));
    const startTime = Date.now();

    try {
      // Simulate contract execution
      await new Promise(resolve => setTimeout(resolve, 2000));

      // Generate mock result
      const mockResult: ResultV1 = {
        ok: true,
        gas_used: Math.floor(Math.random() * executionOptions.gas_limit * 0.1) + 1000,
        output: new TextEncoder().encode(JSON.stringify({ result: "success", processed: true })),
        time: Date.now() - startTime,
        memory: Math.floor(Math.random() * 1024) + 512,
        storage_accesses: [],
        events: [
          {
            name: 'contract.executed',
            data: { contract_id: selectedContract.id, gas_used: 1000 },
            timestamp: new Date().toISOString()
          }
        ],
        timestamp: new Date().toISOString(),
        contract_id: selectedContract.id,
        input_hash: 'mock_input_hash'
      };

      // Add ZK proof if requested
      if (executionOptions.zk_mode) {
        mockResult.proof = {
          algorithm: executionOptions.algorithm,
          proof_data: new Uint8Array([0x01, 0x02, 0x03, 0x04]),
          public_inputs: [new Uint8Array([0x05, 0x06, 0x07, 0x08])],
          circuit_hash: new Uint8Array([0x09, 0x0a, 0x0b, 0x0c]),
          prover_version: '1.0.0',
          proof_size: 64,
          verification_key: new Uint8Array([0x0d, 0x0e, 0x0f, 0x10])
        };
      }

      setExecutionState(prev => ({
        ...prev,
        isExecuting: false,
        result: mockResult,
        executionTime: Date.now() - startTime
      }));

    } catch (error) {
      setExecutionState(prev => ({
        ...prev,
        isExecuting: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      }));
    }
  }, [selectedContract, executionOptions]);

  // Format byte array as hex string
  const formatBytes = (bytes: Uint8Array, maxLength: number = 32): string => {
    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    if (hex.length > maxLength) {
      return hex.substring(0, maxLength) + '...';
    }
    return hex;
  };

  // Format timestamp
  const formatTimestamp = (timestamp: string): string => {
    return new Date(timestamp).toLocaleString();
  };

  // Format gas usage
  const formatGas = (gas: number): string => {
    if (gas >= 1000000) {
      return `${(gas / 1000000).toFixed(1)}M`;
    } else if (gas >= 1000) {
      return `${(gas / 1000).toFixed(1)}K`;
    }
    return gas.toString();
  };

  return (
    <div className="contract-explorer">
      <div className="contract-header">
        <h1>NGFS Smart Contract Explorer</h1>
        <p>Execute and validate smart contracts with gas metering and ZK proofs</p>
      </div>

      <div className="contract-controls">
        <div className="search-filter">
          <input
            type="text"
            placeholder="Search contracts..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
          
          <select
            value={filterCategory}
            onChange={(e) => setFilterCategory(e.target.value as ContractCategory | 'all')}
            className="category-filter"
          >
            <option value="all">All Categories</option>
            {Object.values(ContractCategory).map(category => (
              <option key={category} value={category}>
                {category.charAt(0).toUpperCase() + category.slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div className="sort-controls">
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="sort-select"
          >
            <option value="name">Sort by Name</option>
            <option value="category">Sort by Category</option>
            <option value="created_at">Sort by Created</option>
            <option value="gas_limit">Sort by Gas Limit</option>
          </select>
          
          <button
            onClick={() => setSortOrder(prev => prev === 'asc' ? 'desc' : 'asc')}
            className="sort-order-btn"
          >
            {sortOrder === 'asc' ? '↑' : '↓'}
          </button>
        </div>
      </div>

      <div className="contract-layout">
        <div className="contract-list">
          <h3>Available Contracts ({filteredAndSortedContracts().length})</h3>
          <div className="contract-items">
            {filteredAndSortedContracts().map(contract => (
              <div
                key={contract.id}
                className={`contract-item ${selectedContract?.id === contract.id ? 'selected' : ''}`}
                onClick={() => setSelectedContract(contract)}
              >
                <div className="contract-item-header">
                  <h4>{contract.meta.name}</h4>
                  <span className={`category-badge ${contract.meta.category}`}>
                    {contract.meta.category}
                  </span>
                </div>
                <p className="contract-description">{contract.meta.description}</p>
                <div className="contract-meta">
                  <span>Gas: {formatGas(contract.gas_limit)}</span>
                  <span>ZK: {contract.zk_mode ? '✓' : '✗'}</span>
                  <span>Size: {formatBytes(contract.wasm)}</span>
                </div>
                <div className="contract-tags">
                  {contract.tags.slice(0, 3).map(tag => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                  {contract.tags.length > 3 && (
                    <span className="tag-more">+{contract.tags.length - 3}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="contract-details">
          {selectedContract ? (
            <>
              <div className="contract-info">
                <h3>{selectedContract.meta.name}</h3>
                <p className="contract-description">{selectedContract.meta.description}</p>
                
                <div className="contract-stats">
                  <div className="stat">
                    <label>ID:</label>
                    <span>{selectedContract.id}</span>
                  </div>
                  <div className="stat">
                    <label>Author:</label>
                    <span>{selectedContract.meta.author}</span>
                  </div>
                  <div className="stat">
                    <label>Category:</label>
                    <span className={`category-badge ${selectedContract.meta.category}`}>
                      {selectedContract.meta.category}
                    </span>
                  </div>
                  <div className="stat">
                    <label>Version:</label>
                    <span>{selectedContract.version}</span>
                  </div>
                  <div className="stat">
                    <label>Created:</label>
                    <span>{formatTimestamp(selectedContract.created_at)}</span>
                  </div>
                  <div className="stat">
                    <label>Gas Limit:</label>
                    <span>{formatGas(selectedContract.gas_limit)}</span>
                  </div>
                  <div className="stat">
                    <label>ZK Mode:</label>
                    <span>{selectedContract.zk_mode ? 'Enabled' : 'Disabled'}</span>
                  </div>
                </div>

                <div className="contract-interfaces">
                  <h4>Interfaces</h4>
                  <div className="interface-list">
                    {selectedContract.meta.interfaces.map(iface => (
                      <span key={iface} className="interface">{iface}</span>
                    ))}
                  </div>
                </div>

                <div className="contract-tags">
                  <h4>Tags</h4>
                  <div className="tag-list">
                    {selectedContract.tags.map(tag => (
                      <span key={tag} className="tag">{tag}</span>
                    ))}
                  </div>
                </div>
              </div>

              <div className="execution-panel">
                <h3>Execute Contract</h3>
                
                <div className="execution-options">
                  <div className="option-group">
                    <label>Gas Limit:</label>
                    <input
                      type="number"
                      value={executionOptions.gas_limit}
                      onChange={(e) => setExecutionOptions(prev => ({
                        ...prev,
                        gas_limit: parseInt(e.target.value) || 1000000
                      }))}
                      min="1000"
                      max="10000000"
                      step="1000"
                    />
                  </div>

                  <div className="option-group">
                    <label>ZK Mode:</label>
                    <input
                      type="checkbox"
                      checked={executionOptions.zk_mode}
                      onChange={(e) => setExecutionOptions(prev => ({
                        ...prev,
                        zk_mode: e.target.checked
                      }))}
                    />
                  </div>

                  {executionOptions.zk_mode && (
                    <div className="option-group">
                      <label>ZK Algorithm:</label>
                      <select
                        value={executionOptions.algorithm}
                        onChange={(e) => setExecutionOptions(prev => ({
                          ...prev,
                          algorithm: e.target.value as ZKAlgorithm
                        }))}
                      >
                        <option value={ZKAlgorithm.HALO2}>Halo2</option>
                        <option value={ZKAlgorithm.NOIR}>Noir</option>
                        <option value={ZKAlgorithm.PLONK}>PLONK</option>
                        <option value={ZKAlgorithm.CUSTOM}>Custom</option>
                      </select>
                    </div>
                  )}

                  <div className="option-group">
                    <label>Timeout (ms):</label>
                    <input
                      type="number"
                      value={executionOptions.timeout_ms}
                      onChange={(e) => setExecutionOptions(prev => ({
                        ...prev,
                        timeout_ms: parseInt(e.target.value) || 30000
                      }))}
                      min="1000"
                      max="120000"
                      step="1000"
                    />
                  </div>

                  <div className="option-group">
                    <label>Input Data (JSON):</label>
                    <textarea
                      value={executionOptions.input_data}
                      onChange={(e) => setExecutionOptions(prev => ({
                        ...prev,
                        input_data: e.target.value
                      }))}
                      placeholder='{"key": "value"}'
                      rows={3}
                    />
                  </div>
                </div>

                <button
                  onClick={executeContract}
                  disabled={executionState.isExecuting}
                  className="execute-btn"
                >
                  {executionState.isExecuting ? 'Executing...' : 'Execute Contract'}
                </button>

                {executionState.error && (
                  <div className="execution-error">
                    <h4>Execution Error</h4>
                    <p>{executionState.error}</p>
                  </div>
                )}

                {executionState.result && (
                  <div className="execution-result">
                    <h4>Execution Result</h4>
                    
                    <div className="result-summary">
                      <div className="result-stat">
                        <label>Status:</label>
                        <span className={executionState.result.ok ? 'success' : 'error'}>
                          {executionState.result.ok ? 'Success' : 'Failed'}
                        </span>
                      </div>
                      <div className="result-stat">
                        <label>Gas Used:</label>
                        <span>{formatGas(executionState.result.gas_used)}</span>
                      </div>
                      <div className="result-stat">
                        <label>Execution Time:</label>
                        <span>{executionState.result.time}ms</span>
                      </div>
                      <div className="result-stat">
                        <label>Memory Used:</label>
                        <span>{executionState.result.memory} bytes</span>
                      </div>
                      <div className="result-stat">
                        <label>ZK Proof:</label>
                        <span>{executionState.result.proof ? 'Generated' : 'None'}</span>
                      </div>
                    </div>

                    {executionState.result.output && (
                      <div className="result-output">
                        <h5>Output</h5>
                        <pre>{new TextDecoder().decode(executionState.result.output)}</pre>
                      </div>
                    )}

                    {executionState.result.proof && (
                      <div className="result-proof">
                        <h5>ZK Proof</h5>
                        <div className="proof-details">
                          <div className="proof-stat">
                            <label>Algorithm:</label>
                            <span>{executionState.result.proof.algorithm}</span>
                          </div>
                          <div className="proof-stat">
                            <label>Proof Size:</label>
                            <span>{executionState.result.proof.proof_size} bytes</span>
                          </div>
                          <div className="proof-stat">
                            <label>Prover Version:</label>
                            <span>{executionState.result.proof.prover_version}</span>
                          </div>
                          <div className="proof-stat">
                            <label>Proof Data:</label>
                            <span className="proof-data">{formatBytes(executionState.result.proof.proof_data)}</span>
                          </div>
                        </div>
                      </div>
                    )}

                    {executionState.result.events.length > 0 && (
                      <div className="result-events">
                        <h5>Events</h5>
                        <div className="event-list">
                          {executionState.result.events.map((event, index) => (
                            <div key={index} className="event">
                              <span className="event-name">{event.name}</span>
                              <span className="event-time">{formatTimestamp(event.timestamp)}</span>
                              <pre className="event-data">{JSON.stringify(event.data, null, 2)}</pre>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="no-selection">
              <p>Select a contract to view details and execute</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ContractExplorer;
