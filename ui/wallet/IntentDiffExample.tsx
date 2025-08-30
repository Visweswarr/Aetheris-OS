/**
 * Intent Diff Example Component
 * 
 * Example usage of the IntentDiff component showing various transaction types
 * and how to integrate with the Rust backend summarizer.
 */

import React, { useState } from 'react';
import { IntentDiff } from './IntentDiff';
import type { TransactionIntent, TokenInfo, NetworkInfo, GasEstimate } from './IntentDiff';

// Example token definitions
const USDC: TokenInfo = {
  symbol: 'USDC',
  name: 'USD Coin',
  decimals: 6,
  contractAddress: '0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F',
  chainId: 1,
  logoUrl: 'https://cryptologos.cc/logos/usd-coin-usdc-logo.png',
};

const ETH: TokenInfo = {
  symbol: 'ETH',
  name: 'Ethereum',
  decimals: 18,
  chainId: 1,
  logoUrl: 'https://cryptologos.cc/logos/ethereum-eth-logo.png',
};

const POLY: TokenInfo = {
  symbol: 'POLY',
  name: 'Polymera Token',
  decimals: 18,
  contractAddress: '0x1234567890123456789012345678901234567890',
  chainId: 1,
  logoUrl: 'https://example.com/poly-logo.png',
};

// Example network definitions
const ETHEREUM_MAINNET: NetworkInfo = {
  chainId: 1,
  name: 'Ethereum Mainnet',
  currency: 'ETH',
  blockExplorerUrl: 'https://etherscan.io',
};

const GOERLI_TESTNET: NetworkInfo = {
  chainId: 5,
  name: 'Goerli Testnet',
  currency: 'ETH',
  blockExplorerUrl: 'https://goerli.etherscan.io',
};

// Example gas estimate
const STANDARD_GAS: GasEstimate = {
  gasLimit: 21000,
  gasPrice: '20000000000',
  totalFee: '0.00042',
  feeToken: ETH,
  usdValue: '1.05',
};

const HIGH_GAS: GasEstimate = {
  gasLimit: 150000,
  gasPrice: '50000000000',
  totalFee: '0.0075',
  feeToken: ETH,
  usdValue: '18.75',
};

// Example transaction intents
const EXAMPLE_INTENTS: Record<string, TransactionIntent> = {
  simpleTransfer: {
    operations: [
      {
        type: 'transfer',
        from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
        to: '0x8ba1f109551bD432803012645Hac136c23C6e',
        amount: '1000.50',
        token: USDC,
        memo: 'Payment for services rendered',
      },
    ],
    gasEstimate: STANDARD_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 42,
    metadata: {
      source: 'mobile_app',
      version: '1.0.0',
    },
  },

  highValueTransfer: {
    operations: [
      {
        type: 'transfer',
        from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
        to: '0x8ba1f109551bD432803012645Hac136c23C6e',
        amount: '50000.0',
        token: USDC,
      },
    ],
    gasEstimate: STANDARD_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 43,
    metadata: {},
  },

  tokenSwap: {
    operations: [
      {
        type: 'swap',
        fromToken: USDC,
        toToken: ETH,
        fromAmount: '2000.0',
        minToAmount: '1.2',
        slippageTolerance: '2.5',
        dex: 'Uniswap V3',
      },
    ],
    gasEstimate: HIGH_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 44,
    metadata: {
      dex_version: 'v3',
      pool_fee: 3000,
    },
  },

  unknownContract: {
    operations: [
      {
        type: 'contract_call',
        contractAddress: '0xUnknownContract123456789012345678901234567890',
        functionName: 'claimRewards',
        parameters: [
          {
            name: 'amount',
            paramType: 'uint256',
            value: '1000000000000000000',
          },
          {
            name: 'recipient',
            paramType: 'address',
            value: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
          },
        ],
        value: '0.1',
      },
    ],
    gasEstimate: HIGH_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 45,
    metadata: {},
  },

  multiSigTransaction: {
    operations: [
      {
        type: 'multi_sig',
        multisigAddress: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
        operations: [
          {
            type: 'transfer',
            from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
            to: '0x8ba1f109551bD432803012645Hac136c23C6e',
            amount: '5000.0',
            token: USDC,
          },
          {
            type: 'transfer',
            from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
            to: '0x1234567890123456789012345678901234567890',
            amount: '1.5',
            token: ETH,
          },
        ],
        requiredSignatures: 3,
        currentSignatures: 2,
      },
    ],
    gasEstimate: HIGH_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 46,
    metadata: {
      multisig_type: 'gnosis_safe',
    },
  },

  stakingOperation: {
    operations: [
      {
        type: 'stake',
        validator: '0x1234567890123456789012345678901234567890',
        amount: '32.0',
        token: ETH,
        lockPeriod: '365 days',
      },
    ],
    gasEstimate: STANDARD_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 47,
    metadata: {
      validator_name: 'Polymera Validator 1',
    },
  },

  defiOperation: {
    operations: [
      {
        type: 'defi_operation',
        protocol: 'compound',
        operationType: 'Supply',
        tokens: [
          {
            token: USDC,
            amount: '10000.0',
            usdValue: '10000.00',
          },
        ],
        parameters: {
          market: 'cUSDC',
          apy: '3.5%',
        },
      },
    ],
    gasEstimate: HIGH_GAS,
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 48,
    metadata: {},
  },

  testnetTransaction: {
    operations: [
      {
        type: 'transfer',
        from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
        to: '0x8ba1f109551bD432803012645Hac136c23C6e',
        amount: '100.0',
        token: USDC,
        memo: 'Test transaction',
      },
    ],
    gasEstimate: STANDARD_GAS,
    network: GOERLI_TESTNET,
    timestamp: new Date().toISOString(),
    nonce: 49,
    metadata: {
      environment: 'test',
    },
  },

  complexMultiOperation: {
    operations: [
      {
        type: 'swap',
        fromToken: USDC,
        toToken: ETH,
        fromAmount: '1000.0',
        minToAmount: '0.6',
        slippageTolerance: '1.0',
        dex: 'Uniswap V3',
      },
      {
        type: 'stake',
        validator: '0x1234567890123456789012345678901234567890',
        amount: '0.5',
        token: ETH,
        lockPeriod: '30 days',
      },
      {
        type: 'transfer',
        from: '0x742d35Cc1fF1c0B5c1EFEB83D9A6b7C9F9FC3A8B',
        to: '0x8ba1f109551bD432803012645Hac136c23C6e',
        amount: '100.0',
        token: POLY,
      },
    ],
    gasEstimate: {
      gasLimit: 300000,
      gasPrice: '30000000000',
      totalFee: '0.009',
      feeToken: ETH,
      usdValue: '22.50',
    },
    network: ETHEREUM_MAINNET,
    timestamp: new Date().toISOString(),
    nonce: 50,
    metadata: {
      complexity: 'high',
      batch_operation: true,
    },
  },
};

// Example component
export const IntentDiffExample: React.FC = () => {
  const [selectedExample, setSelectedExample] = useState<string>('simpleTransfer');
  const [theme, setTheme] = useState<'light' | 'dark'>('light');
  const [showDetails, setShowDetails] = useState(false);
  const [confirmed, setConfirmed] = useState<string | null>(null);
  const [rejected, setRejected] = useState<string | null>(null);

  const handleConfirm = () => {
    setConfirmed(selectedExample);
    setRejected(null);
    setTimeout(() => setConfirmed(null), 3000);
  };

  const handleReject = () => {
    setRejected(selectedExample);
    setConfirmed(null);
    setTimeout(() => setRejected(null), 3000);
  };

  const currentIntent = EXAMPLE_INTENTS[selectedExample];

  return (
    <div className={`min-h-screen p-8 transition-colors ${
      theme === 'light' ? 'bg-gray-50' : 'bg-gray-900'
    }`}>
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className={`text-center ${theme === 'light' ? 'text-gray-900' : 'text-gray-100'}`}>
          <h1 className="text-3xl font-bold mb-2">Intent Diff Example</h1>
          <p className="text-lg text-gray-600 dark:text-gray-400">
            Explore different transaction types and see how they're explained in natural language
          </p>
        </div>

        {/* Controls */}
        <div className={`p-6 rounded-xl shadow-lg ${
          theme === 'light' ? 'bg-white border border-gray-200' : 'bg-gray-800 border border-gray-700'
        }`}>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Example Selection */}
            <div>
              <label className={`block text-sm font-medium mb-2 ${
                theme === 'light' ? 'text-gray-700' : 'text-gray-300'
              }`}>
                Example Transaction
              </label>
              <select
                value={selectedExample}
                onChange={(e) => setSelectedExample(e.target.value)}
                className={`w-full p-2 border rounded-lg ${
                  theme === 'light' 
                    ? 'border-gray-300 bg-white text-gray-900' 
                    : 'border-gray-600 bg-gray-700 text-gray-100'
                }`}
              >
                <option value="simpleTransfer">Simple Transfer</option>
                <option value="highValueTransfer">High-Value Transfer</option>
                <option value="tokenSwap">Token Swap</option>
                <option value="unknownContract">Unknown Contract Call</option>
                <option value="multiSigTransaction">MultiSig Transaction</option>
                <option value="stakingOperation">Staking Operation</option>
                <option value="defiOperation">DeFi Operation</option>
                <option value="testnetTransaction">Testnet Transaction</option>
                <option value="complexMultiOperation">Complex Multi-Operation</option>
              </select>
            </div>

            {/* Theme Selection */}
            <div>
              <label className={`block text-sm font-medium mb-2 ${
                theme === 'light' ? 'text-gray-700' : 'text-gray-300'
              }`}>
                Theme
              </label>
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value as 'light' | 'dark')}
                className={`w-full p-2 border rounded-lg ${
                  theme === 'light' 
                    ? 'border-gray-300 bg-white text-gray-900' 
                    : 'border-gray-600 bg-gray-700 text-gray-100'
                }`}
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>

            {/* Options */}
            <div>
              <label className={`block text-sm font-medium mb-2 ${
                theme === 'light' ? 'text-gray-700' : 'text-gray-300'
              }`}>
                Options
              </label>
              <div className="space-y-2">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={showDetails}
                    onChange={(e) => setShowDetails(e.target.checked)}
                    className="mr-2"
                  />
                  <span className={`text-sm ${
                    theme === 'light' ? 'text-gray-600' : 'text-gray-400'
                  }`}>
                    Show Details by Default
                  </span>
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Status Messages */}
        {confirmed && (
          <div className={`p-4 rounded-lg border-l-4 border-green-500 ${
            theme === 'light' ? 'bg-green-50 text-green-800' : 'bg-green-900/20 text-green-300'
          }`}>
            <p className="font-medium">✅ Transaction Confirmed!</p>
            <p className="text-sm">Example: {selectedExample}</p>
          </div>
        )}

        {rejected && (
          <div className={`p-4 rounded-lg border-l-4 border-red-500 ${
            theme === 'light' ? 'bg-red-50 text-red-800' : 'bg-red-900/20 text-red-300'
          }`}>
            <p className="font-medium">❌ Transaction Rejected!</p>
            <p className="text-sm">Example: {selectedExample}</p>
          </div>
        )}

        {/* Intent Diff Component */}
        <IntentDiff
          intent={currentIntent}
          onConfirm={handleConfirm}
          onReject={handleReject}
          theme={theme}
          showDetails={showDetails}
          className="shadow-2xl"
        />

        {/* Raw Intent Data (for debugging) */}
        <details className={`p-4 rounded-lg ${
          theme === 'light' ? 'bg-gray-100' : 'bg-gray-800'
        }`}>
          <summary className={`cursor-pointer font-medium ${
            theme === 'light' ? 'text-gray-700' : 'text-gray-300'
          }`}>
            View Raw Intent Data
          </summary>
          <pre className={`mt-4 p-4 rounded border text-xs overflow-x-auto ${
            theme === 'light' 
              ? 'bg-white border-gray-300 text-gray-800' 
              : 'bg-gray-900 border-gray-600 text-gray-200'
          }`}>
            {JSON.stringify(currentIntent, null, 2)}
          </pre>
        </details>

        {/* Documentation */}
        <div className={`p-6 rounded-xl ${
          theme === 'light' ? 'bg-blue-50 border border-blue-200' : 'bg-blue-900/20 border border-blue-700'
        }`}>
          <h3 className={`text-lg font-semibold mb-4 ${
            theme === 'light' ? 'text-blue-900' : 'text-blue-300'
          }`}>
            How It Works
          </h3>
          <div className={`space-y-3 text-sm ${
            theme === 'light' ? 'text-blue-800' : 'text-blue-200'
          }`}>
            <p>
              <strong>1. Intent Analysis:</strong> The Rust backend analyzes the transaction intent structure, 
              identifying operation types, amounts, addresses, and protocols.
            </p>
            <p>
              <strong>2. Natural Language Generation:</strong> Complex blockchain operations are converted 
              into human-readable explanations like "Send 1000 USDC to Alice" or "Swap ETH for USDC on Uniswap".
            </p>
            <p>
              <strong>3. Risk Assessment:</strong> The system evaluates transaction risk based on factors like 
              contract verification, transaction amounts, and operation complexity.
            </p>
            <p>
              <strong>4. User Interface:</strong> The React component presents the information clearly with 
              appropriate warnings, details on demand, and confirmation controls.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default IntentDiffExample;
