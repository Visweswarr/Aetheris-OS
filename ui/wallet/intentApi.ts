/**
 * Intent API Integration
 * 
 * Provides API functions to communicate with the Rust intent summarizer backend.
 * Supports both WebAssembly and HTTP API integration patterns.
 */

import type { TransactionIntent, IntentSummary } from './IntentDiff';

// API configuration
interface ApiConfig {
  baseUrl?: string;
  timeout?: number;
  retries?: number;
  wasmEnabled?: boolean;
}

const DEFAULT_CONFIG: ApiConfig = {
  baseUrl: '/api/wallet',
  timeout: 30000, // 30 seconds
  retries: 3,
  wasmEnabled: false, // Will be enabled when WASM module is loaded
};

// Error types
export class IntentApiError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public details?: any
  ) {
    super(message);
    this.name = 'IntentApiError';
  }
}

// WASM module interface (loaded dynamically)
interface WasmModule {
  summarize_intent: (intentJson: string) => string;
  memory: WebAssembly.Memory;
}

let wasmModule: WasmModule | null = null;
let wasmLoadPromise: Promise<WasmModule> | null = null;

/**
 * Load the WASM module for client-side intent summarization
 */
export async function loadWasmModule(): Promise<WasmModule> {
  if (wasmModule) {
    return wasmModule;
  }

  if (wasmLoadPromise) {
    return wasmLoadPromise;
  }

  wasmLoadPromise = (async () => {
    try {
      // Load the WASM module (adjust path as needed)
      const wasmResponse = await fetch('/wasm/intent_summarizer.wasm');
      if (!wasmResponse.ok) {
        throw new Error(`Failed to load WASM: ${wasmResponse.statusText}`);
      }

      const wasmBytes = await wasmResponse.arrayBuffer();
      const wasmModule = await WebAssembly.instantiate(wasmBytes, {
        // Import object for WASM if needed
        env: {
          memory: new WebAssembly.Memory({ initial: 256, maximum: 256 }),
        },
      });

      const module = wasmModule.instance.exports as any;
      
      // Verify required exports exist
      if (!module.summarize_intent) {
        throw new Error('WASM module missing summarize_intent export');
      }

      return module;
    } catch (error) {
      console.warn('Failed to load WASM module:', error);
      throw new IntentApiError('WASM module loading failed', 0, error);
    }
  })();

  try {
    wasmModule = await wasmLoadPromise;
    return wasmModule;
  } catch (error) {
    wasmLoadPromise = null;
    throw error;
  }
}

/**
 * Check if WASM is available and loaded
 */
export function isWasmAvailable(): boolean {
  return wasmModule !== null;
}

/**
 * Summarize intent using WASM (client-side)
 */
async function summarizeIntentWasm(intent: TransactionIntent): Promise<IntentSummary> {
  const module = await loadWasmModule();
  
  try {
    const intentJson = JSON.stringify(intent);
    const resultJson = module.summarize_intent(intentJson);
    const summary = JSON.parse(resultJson);
    
    return summary;
  } catch (error) {
    throw new IntentApiError('WASM summarization failed', 0, error);
  }
}

/**
 * Summarize intent using HTTP API (server-side)
 */
async function summarizeIntentHttp(
  intent: TransactionIntent,
  config: ApiConfig = {}
): Promise<IntentSummary> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  const url = `${finalConfig.baseUrl}/summarize-intent`;
  
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), finalConfig.timeout);

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      body: JSON.stringify(intent),
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      const errorText = await response.text().catch(() => 'Unknown error');
      throw new IntentApiError(
        `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorText
      );
    }

    const summary: IntentSummary = await response.json();
    return summary;
  } catch (error) {
    clearTimeout(timeoutId);
    
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new IntentApiError('Request timeout', 408);
    }
    
    if (error instanceof IntentApiError) {
      throw error;
    }
    
    throw new IntentApiError('Network request failed', 0, error);
  }
}

/**
 * Summarize intent with automatic fallback from WASM to HTTP
 */
export async function summarizeIntent(
  intent: TransactionIntent,
  config: ApiConfig = {}
): Promise<IntentSummary> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  
  // Try WASM first if enabled and available
  if (finalConfig.wasmEnabled && isWasmAvailable()) {
    try {
      return await summarizeIntentWasm(intent);
    } catch (error) {
      console.warn('WASM summarization failed, falling back to HTTP:', error);
      // Fall through to HTTP API
    }
  }

  // Use HTTP API with retries
  let lastError: Error;
  const maxRetries = finalConfig.retries || 1;
  
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await summarizeIntentHttp(intent, finalConfig);
    } catch (error) {
      lastError = error as Error;
      
      // Don't retry certain errors
      if (error instanceof IntentApiError) {
        if (error.statusCode && error.statusCode >= 400 && error.statusCode < 500) {
          // Client errors (4xx) - don't retry
          throw error;
        }
      }
      
      // Wait before retry (exponential backoff)
      if (attempt < maxRetries - 1) {
        const delay = Math.min(1000 * Math.pow(2, attempt), 10000);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  
  throw lastError!;
}

/**
 * Validate a transaction intent structure
 */
export function validateIntent(intent: TransactionIntent): string[] {
  const errors: string[] = [];
  
  if (!intent.operations || intent.operations.length === 0) {
    errors.push('Intent must contain at least one operation');
  }
  
  if (!intent.network) {
    errors.push('Intent must specify a network');
  } else {
    if (!intent.network.chainId || intent.network.chainId <= 0) {
      errors.push('Network must have a valid chain ID');
    }
    if (!intent.network.name || intent.network.name.trim() === '') {
      errors.push('Network must have a name');
    }
  }
  
  if (!intent.timestamp) {
    errors.push('Intent must have a timestamp');
  } else {
    try {
      const date = new Date(intent.timestamp);
      if (isNaN(date.getTime())) {
        errors.push('Intent timestamp must be a valid ISO date string');
      }
    } catch {
      errors.push('Intent timestamp must be a valid ISO date string');
    }
  }
  
  // Validate operations
  intent.operations.forEach((operation, index) => {
    const prefix = `Operation ${index + 1}`;
    
    switch (operation.type) {
      case 'transfer':
        if (!operation.from || !operation.to) {
          errors.push(`${prefix}: Transfer must have from and to addresses`);
        }
        if (!operation.amount || parseFloat(operation.amount) <= 0) {
          errors.push(`${prefix}: Transfer must have a positive amount`);
        }
        if (!operation.token || !operation.token.symbol) {
          errors.push(`${prefix}: Transfer must specify a token`);
        }
        break;
        
      case 'swap':
        if (!operation.fromToken || !operation.toToken) {
          errors.push(`${prefix}: Swap must have from and to tokens`);
        }
        if (!operation.fromAmount || parseFloat(operation.fromAmount) <= 0) {
          errors.push(`${prefix}: Swap must have a positive from amount`);
        }
        if (!operation.minToAmount || parseFloat(operation.minToAmount) <= 0) {
          errors.push(`${prefix}: Swap must have a positive minimum to amount`);
        }
        break;
        
      case 'contract_call':
        if (!operation.contractAddress) {
          errors.push(`${prefix}: Contract call must specify contract address`);
        }
        if (!operation.functionName) {
          errors.push(`${prefix}: Contract call must specify function name`);
        }
        break;
        
      case 'multi_sig':
        if (!operation.multisigAddress) {
          errors.push(`${prefix}: MultiSig must specify multisig address`);
        }
        if (!operation.operations || operation.operations.length === 0) {
          errors.push(`${prefix}: MultiSig must contain inner operations`);
        }
        if (operation.requiredSignatures <= 0) {
          errors.push(`${prefix}: MultiSig must require at least 1 signature`);
        }
        break;
        
      case 'stake':
      case 'unstake':
        if (!operation.validator) {
          errors.push(`${prefix}: Staking operation must specify validator`);
        }
        if (!operation.amount || parseFloat(operation.amount) <= 0) {
          errors.push(`${prefix}: Staking operation must have a positive amount`);
        }
        if (!operation.token) {
          errors.push(`${prefix}: Staking operation must specify a token`);
        }
        break;
        
      case 'defi_operation':
        if (!operation.protocol) {
          errors.push(`${prefix}: DeFi operation must specify protocol`);
        }
        if (!operation.operationType) {
          errors.push(`${prefix}: DeFi operation must specify operation type`);
        }
        break;
        
      default:
        errors.push(`${prefix}: Unknown operation type: ${(operation as any).type}`);
    }
  });
  
  return errors;
}

/**
 * Create a mock intent summarizer for development/testing
 */
export function createMockSummarizer(): (intent: TransactionIntent) => Promise<IntentSummary> {
  return async (intent: TransactionIntent): Promise<IntentSummary> => {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 1000));
    
    const operation = intent.operations[0];
    let primaryAction = 'Unknown operation';
    let riskLevel: IntentSummary['riskLevel'] = 'Medium';
    let confidence = 0.8;
    const warnings: string[] = [];
    
    // Generate mock summary based on operation type
    switch (operation.type) {
      case 'transfer':
        primaryAction = `Send ${operation.amount} ${operation.token.symbol} to ${operation.to.slice(0, 6)}...${operation.to.slice(-4)}`;
        riskLevel = 'Low';
        confidence = 0.95;
        
        if (parseFloat(operation.amount) > 10000) {
          warnings.push('💰 High-value transfer detected');
        }
        break;
        
      case 'swap':
        primaryAction = `Swap ${operation.fromAmount} ${operation.fromToken.symbol} for ${operation.minToAmount} ${operation.toToken.symbol}`;
        riskLevel = 'Medium';
        confidence = 0.9;
        break;
        
      case 'contract_call':
        primaryAction = `Call ${operation.functionName} on ${operation.contractAddress.slice(0, 6)}...${operation.contractAddress.slice(-4)}`;
        riskLevel = 'High';
        confidence = 0.6;
        warnings.push('⚠️ Interacting with unverified contract');
        break;
        
      case 'multi_sig':
        primaryAction = `Execute ${operation.requiredSignatures}-of-${operation.currentSignatures} multisig with ${operation.operations.length} operations`;
        riskLevel = 'Medium';
        confidence = 0.8;
        break;
        
      case 'stake':
        primaryAction = `Stake ${operation.amount} ${operation.token.symbol} with validator`;
        riskLevel = 'Medium';
        confidence = 0.85;
        break;
        
      case 'unstake':
        primaryAction = `Unstake ${operation.amount} ${operation.token.symbol} from validator`;
        riskLevel = 'Medium';
        confidence = 0.85;
        break;
        
      case 'defi_operation':
        primaryAction = `${operation.operationType} on ${operation.protocol}`;
        riskLevel = 'Medium';
        confidence = 0.75;
        break;
    }
    
    // Multiple operations
    if (intent.operations.length > 1) {
      primaryAction = `Execute ${intent.operations.length} operations: ${intent.operations.map(op => op.type).join(', ')}`;
      riskLevel = 'Medium';
      confidence = Math.max(0.7, confidence - 0.1);
    }
    
    // Testnet warning
    if (intent.network.chainId !== 1) {
      warnings.push('ℹ️ This transaction will execute on a test network');
    }
    
    return {
      primaryAction,
      details: intent.operations.map((op, i) => `${i + 1}. ${op.type} operation`),
      warnings,
      networkInfo: `Network: ${intent.network.name} (Chain ID: ${intent.network.chainId})`,
      feeSummary: intent.gasEstimate ? 
        `Estimated fee: ${intent.gasEstimate.totalFee} ${intent.gasEstimate.feeToken.symbol}${
          intent.gasEstimate.usdValue ? ` (~$${intent.gasEstimate.usdValue})` : ''
        }` : undefined,
      riskLevel,
      confidence,
    };
  };
}

// Export default instance
export const intentApi = {
  summarizeIntent,
  validateIntent,
  loadWasmModule,
  isWasmAvailable,
  createMockSummarizer,
};
