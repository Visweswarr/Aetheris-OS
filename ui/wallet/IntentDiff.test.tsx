/**
 * Intent Diff Component Tests
 * 
 * Snapshot tests and unit tests for the IntentDiff React component,
 * verifying UI rendering and interaction behavior.
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { IntentDiff, useIntentSummary, formatAddress, getRiskLevelStyles } from './IntentDiff';
import type { TransactionIntent, TokenInfo, NetworkInfo, GasEstimate, IntentSummary } from './IntentDiff';

// Mock framer-motion to avoid animation issues in tests
jest.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

// Mock fetch for API calls
global.fetch = jest.fn();

describe('IntentDiff Component', () => {
  // Test data factories
  const createTestToken = (): TokenInfo => ({
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    contractAddress: '0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F',
    chainId: 1,
  });

  const createTestNetwork = (): NetworkInfo => ({
    chainId: 1,
    name: 'Ethereum Mainnet',
    currency: 'ETH',
    blockExplorerUrl: 'https://etherscan.io',
  });

  const createTestGasEstimate = (): GasEstimate => ({
    gasLimit: 21000,
    gasPrice: '20000000000',
    totalFee: '0.00042',
    feeToken: {
      symbol: 'ETH',
      name: 'Ethereum',
      decimals: 18,
    },
    usdValue: '1.05',
  });

  const createTestIntent = (): TransactionIntent => ({
    operations: [
      {
        type: 'transfer',
        from: '0x1234567890123456789012345678901234567890',
        to: '0x0987654321098765432109876543210987654321',
        amount: '1000.50',
        token: createTestToken(),
        memo: 'Payment for services',
      },
    ],
    gasEstimate: createTestGasEstimate(),
    network: createTestNetwork(),
    timestamp: '2024-01-15T14:00:00Z',
    nonce: 42,
    metadata: {},
  });

  const createTestSummary = (): IntentSummary => ({
    primaryAction: 'Send 1000.50 USDC to 0x0987...4321 with memo: \'Payment for services\'',
    details: ['Transfer 1000.50 USDC from 0x1234...7890 to 0x0987...4321'],
    warnings: [],
    networkInfo: 'Network: Ethereum Mainnet (Chain ID: 1)',
    feeSummary: 'Estimated fee: 0.00042 ETH (~$1.05)',
    riskLevel: 'Low',
    confidence: 0.95,
  });

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Component Rendering', () => {
    it('should render loading state', () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValue(new Promise(() => {})); // Never resolves

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      expect(screen.getByText('Transaction Summary')).toBeInTheDocument();
      // Should show loading skeleton
      expect(document.querySelector('.animate-pulse')).toBeInTheDocument();
    });

    it('should render successful summary', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Send 1000.50 USDC to 0x0987...4321 with memo: \'Payment for services\'')).toBeInTheDocument();
      });

      expect(screen.getByText('Low Risk')).toBeInTheDocument();
      expect(screen.getByText('95% Confidence')).toBeInTheDocument();
      expect(screen.getByText('Network: Ethereum Mainnet (Chain ID: 1)')).toBeInTheDocument();
      expect(screen.getByText('Estimated fee: 0.00042 ETH (~$1.05)')).toBeInTheDocument();
    });

    it('should render error state', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockRejectedValueOnce(new Error('API Error'));

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Failed to summarize transaction')).toBeInTheDocument();
      });

      expect(screen.getByText('API Error')).toBeInTheDocument();
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    it('should show warnings when present', async () => {
      const summaryWithWarnings: IntentSummary = {
        ...createTestSummary(),
        warnings: ['⚠️ High-value transfer: 50000.0 USDC', '⚠️ Interacting with unverified contract'],
        riskLevel: 'High',
      };

      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => summaryWithWarnings,
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Important Warnings')).toBeInTheDocument();
      });

      expect(screen.getByText('⚠️ High-value transfer: 50000.0 USDC')).toBeInTheDocument();
      expect(screen.getByText('⚠️ Interacting with unverified contract')).toBeInTheDocument();
      expect(screen.getByText('High Risk')).toBeInTheDocument();
    });
  });

  describe('User Interactions', () => {
    it('should call onConfirm when confirm button is clicked', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const onConfirm = jest.fn();
      const onReject = jest.fn();

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={onConfirm}
          onReject={onReject}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Confirm & Sign')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('Confirm & Sign'));
      expect(onConfirm).toHaveBeenCalledTimes(1);
      expect(onReject).not.toHaveBeenCalled();
    });

    it('should call onReject when reject button is clicked', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const onConfirm = jest.fn();
      const onReject = jest.fn();

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={onConfirm}
          onReject={onReject}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Reject Transaction')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('Reject Transaction'));
      expect(onReject).toHaveBeenCalledTimes(1);
      expect(onConfirm).not.toHaveBeenCalled();
    });

    it('should show "Sign Anyway" for critical risk transactions', async () => {
      const criticalSummary: IntentSummary = {
        ...createTestSummary(),
        riskLevel: 'Critical',
        warnings: ['⚠️ Critical: Interacting with known malicious contract'],
      };

      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => criticalSummary,
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Sign Anyway')).toBeInTheDocument();
      });

      expect(screen.getByText('Critical Risk')).toBeInTheDocument();
    });

    it('should toggle details visibility', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Show Details')).toBeInTheDocument();
      });

      // Initially details should be hidden
      expect(screen.queryByText('Operation Details')).not.toBeInTheDocument();

      // Click to show details
      fireEvent.click(screen.getByText('Show Details'));
      expect(screen.getByText('Operation Details')).toBeInTheDocument();
      expect(screen.getByText('Hide Details')).toBeInTheDocument();

      // Click to hide details
      fireEvent.click(screen.getByText('Hide Details'));
      expect(screen.queryByText('Operation Details')).not.toBeInTheDocument();
    });

    it('should toggle raw data visibility', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
          showDetails={true}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Show Raw Transaction Data')).toBeInTheDocument();
      });

      // Click to show raw data
      fireEvent.click(screen.getByText('Show Raw Transaction Data'));
      expect(screen.getByText('Hide Raw Transaction Data')).toBeInTheDocument();
      
      // Should show JSON content
      const jsonContent = screen.getByText(/"operations"/);
      expect(jsonContent).toBeInTheDocument();
    });
  });

  describe('Theme Support', () => {
    it('should apply light theme styles', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const { container } = render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
          theme="light"
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Transaction Summary')).toBeInTheDocument();
      });

      // Check for light theme classes
      expect(container.querySelector('.bg-white')).toBeInTheDocument();
      expect(container.querySelector('.text-gray-900')).toBeInTheDocument();
    });

    it('should apply dark theme styles', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const { container } = render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
          theme="dark"
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Transaction Summary')).toBeInTheDocument();
      });

      // Check for dark theme classes
      expect(container.querySelector('.bg-gray-900')).toBeInTheDocument();
      expect(container.querySelector('.text-gray-100')).toBeInTheDocument();
    });
  });

  describe('API Integration', () => {
    it('should call the correct API endpoint', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith('/api/wallet/summarize-intent', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(createTestIntent()),
        });
      });
    });

    it('should handle API errors gracefully', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: 'Internal Server Error',
      } as Response);

      render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Failed to summarize intent: Internal Server Error')).toBeInTheDocument();
      });
    });
  });

  describe('Utility Functions', () => {
    describe('formatAddress', () => {
      it('should format long addresses correctly', () => {
        const longAddress = '0x1234567890123456789012345678901234567890';
        expect(formatAddress(longAddress)).toBe('0x1234...7890');
      });

      it('should return short addresses unchanged', () => {
        const shortAddress = '0x1234';
        expect(formatAddress(shortAddress)).toBe('0x1234');
      });
    });

    describe('getRiskLevelStyles', () => {
      it('should return correct styles for each risk level', () => {
        expect(getRiskLevelStyles('Low', 'light')).toContain('bg-green-100');
        expect(getRiskLevelStyles('Medium', 'light')).toContain('bg-yellow-100');
        expect(getRiskLevelStyles('High', 'light')).toContain('bg-orange-100');
        expect(getRiskLevelStyles('Critical', 'light')).toContain('bg-red-100');
      });

      it('should return correct dark theme styles', () => {
        expect(getRiskLevelStyles('Low', 'dark')).toContain('bg-green-900/30');
        expect(getRiskLevelStyles('Medium', 'dark')).toContain('bg-yellow-900/30');
        expect(getRiskLevelStyles('High', 'dark')).toContain('bg-orange-900/30');
        expect(getRiskLevelStyles('Critical', 'dark')).toContain('bg-red-900/30');
      });
    });
  });

  describe('useIntentSummary Hook', () => {
    it('should return loading state initially', () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValue(new Promise(() => {})); // Never resolves

      const TestComponent = () => {
        const { summary, loading, error } = useIntentSummary(createTestIntent());
        return (
          <div>
            <div data-testid="loading">{loading.toString()}</div>
            <div data-testid="summary">{summary ? 'has summary' : 'no summary'}</div>
            <div data-testid="error">{error || 'no error'}</div>
          </div>
        );
      };

      render(<TestComponent />);

      expect(screen.getByTestId('loading')).toHaveTextContent('true');
      expect(screen.getByTestId('summary')).toHaveTextContent('no summary');
      expect(screen.getByTestId('error')).toHaveTextContent('no error');
    });

    it('should return summary on successful fetch', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const TestComponent = () => {
        const { summary, loading, error } = useIntentSummary(createTestIntent());
        return (
          <div>
            <div data-testid="loading">{loading.toString()}</div>
            <div data-testid="summary">{summary ? 'has summary' : 'no summary'}</div>
            <div data-testid="error">{error || 'no error'}</div>
          </div>
        );
      };

      render(<TestComponent />);

      await waitFor(() => {
        expect(screen.getByTestId('loading')).toHaveTextContent('false');
        expect(screen.getByTestId('summary')).toHaveTextContent('has summary');
        expect(screen.getByTestId('error')).toHaveTextContent('no error');
      });
    });

    it('should return error on failed fetch', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const TestComponent = () => {
        const { summary, loading, error } = useIntentSummary(createTestIntent());
        return (
          <div>
            <div data-testid="loading">{loading.toString()}</div>
            <div data-testid="summary">{summary ? 'has summary' : 'no summary'}</div>
            <div data-testid="error">{error || 'no error'}</div>
          </div>
        );
      };

      render(<TestComponent />);

      await waitFor(() => {
        expect(screen.getByTestId('loading')).toHaveTextContent('false');
        expect(screen.getByTestId('summary')).toHaveTextContent('no summary');
        expect(screen.getByTestId('error')).toHaveTextContent('Network error');
      });
    });
  });

  describe('Snapshot Tests', () => {
    it('should match snapshot for simple transfer', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createTestSummary(),
      } as Response);

      const { container } = render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Transaction Summary')).toBeInTheDocument();
      });

      expect(container.firstChild).toMatchSnapshot();
    });

    it('should match snapshot for high-risk transaction', async () => {
      const highRiskSummary: IntentSummary = {
        ...createTestSummary(),
        primaryAction: 'Call unknownFunction on 0xUnkn...7890 with 1.0 ETH',
        riskLevel: 'Critical',
        confidence: 0.45,
        warnings: ['⚠️ Interacting with unverified contract', '💰 High-value transaction'],
      };

      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => highRiskSummary,
      } as Response);

      const { container } = render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Critical Risk')).toBeInTheDocument();
      });

      expect(container.firstChild).toMatchSnapshot();
    });

    it('should match snapshot for error state', async () => {
      const mockFetch = jest.mocked(global.fetch);
      mockFetch.mockRejectedValueOnce(new Error('Failed to parse intent'));

      const { container } = render(
        <IntentDiff
          intent={createTestIntent()}
          onConfirm={jest.fn()}
          onReject={jest.fn()}
        />
      );

      await waitFor(() => {
        expect(screen.getByText('Failed to summarize transaction')).toBeInTheDocument();
      });

      expect(container.firstChild).toMatchSnapshot();
    });
  });
});

// Integration test using both components together
describe('IntentDiff Integration', () => {
  it('should handle complex multi-operation transaction', async () => {
    const complexIntent: TransactionIntent = {
      operations: [
        {
          type: 'swap',
          fromToken: { symbol: 'USDC', name: 'USD Coin', decimals: 6 },
          toToken: { symbol: 'ETH', name: 'Ethereum', decimals: 18 },
          fromAmount: '1000.0',
          minToAmount: '0.5',
          slippageTolerance: '2.0',
          dex: 'Uniswap',
        },
        {
          type: 'transfer',
          from: '0x1111111111111111111111111111111111111111',
          to: '0x2222222222222222222222222222222222222222',
          amount: '0.25',
          token: { symbol: 'ETH', name: 'Ethereum', decimals: 18 },
        },
      ],
      gasEstimate: createTestGasEstimate(),
      network: createTestNetwork(),
      timestamp: '2024-01-15T14:00:00Z',
      nonce: 42,
      metadata: {},
    };

    const complexSummary: IntentSummary = {
      primaryAction: 'Execute 2 operations: Swap, Transfer',
      details: [
        '1. You will give: 1000.00 USDC, You will receive: at least 0.50 ETH, Maximum slippage: 2.0%, Exchange: Uniswap',
        '2. Transfer 0.25 ETH from 0x1111...1111 to 0x2222...2222',
      ],
      warnings: [],
      networkInfo: 'Network: Ethereum Mainnet (Chain ID: 1)',
      feeSummary: 'Estimated fee: 0.00042 ETH (~$1.05)',
      riskLevel: 'Medium',
      confidence: 0.85,
    };

    const mockFetch = jest.mocked(global.fetch);
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => complexSummary,
    } as Response);

    render(
      <IntentDiff
        intent={complexIntent}
        onConfirm={jest.fn()}
        onReject={jest.fn()}
        showDetails={true}
      />
    );

    await waitFor(() => {
      expect(screen.getByText('Execute 2 operations: Swap, Transfer')).toBeInTheDocument();
    });

    expect(screen.getByText('Medium Risk')).toBeInTheDocument();
    expect(screen.getByText('85% Confidence')).toBeInTheDocument();
    expect(screen.getByText('Operation Details')).toBeInTheDocument();
    
    // Check that both operation details are shown
    expect(screen.getByText(/You will give: 1000.00 USDC/)).toBeInTheDocument();
    expect(screen.getByText(/Transfer 0.25 ETH/)).toBeInTheDocument();
  });
});
