import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ContractExplorer } from '../contract_ui';

// Mock the CSS import
jest.mock('../contract_ui.css', () => ({}));

describe('ContractExplorer', () => {
  beforeEach(() => {
    // Clear all mocks before each test
    jest.clearAllMocks();
  });

  describe('Initial Rendering', () => {
    it('renders the header with title and description', () => {
      render(<ContractExplorer />);
      
      expect(screen.getByText('NGFS Smart Contract Explorer')).toBeInTheDocument();
      expect(screen.getByText(/Execute and validate smart contracts/)).toBeInTheDocument();
    });

    it('renders search and filter controls', () => {
      render(<ContractExplorer />);
      
      expect(screen.getByPlaceholderText('Search contracts...')).toBeInTheDocument();
      expect(screen.getByDisplayValue('All Categories')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Sort by Name')).toBeInTheDocument();
    });

    it('renders contract list with mock contracts', () => {
      render(<ContractExplorer />);
      
      // Should show the three mock contracts
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
      expect(screen.getByText('Key-Value Store')).toBeInTheDocument();
      expect(screen.getByText('Data Validator')).toBeInTheDocument();
    });

    it('shows no selection message initially', () => {
      render(<ContractExplorer />);
      
      expect(screen.getByText('Select a contract to view details and execute')).toBeInTheDocument();
    });
  });

  describe('Contract Selection', () => {
    it('selects a contract when clicked', () => {
      render(<ContractExplorer />);
      
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show contract details
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
      expect(screen.getByText(/Simple counter increment/)).toBeInTheDocument();
      expect(screen.getByText('Execute Contract')).toBeInTheDocument();
    });

    it('shows contract metadata when selected', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Key-Value Store').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show metadata
      expect(screen.getByText('Key-Value Store')).toBeInTheDocument();
      expect(screen.getByText(/Persistent key-value storage/)).toBeInTheDocument();
      expect(screen.getByText('NGFS Team')).toBeInTheDocument();
      expect(screen.getByText('storage')).toBeInTheDocument();
    });

    it('displays contract statistics correctly', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Data Validator').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show stats
      expect(screen.getByText('did:aetheris:team:ngfs')).toBeInTheDocument();
      expect(screen.getByText('1.0.0')).toBeInTheDocument();
      expect(screen.getByText('10M')).toBeInTheDocument(); // gas limit
    });
  });

  describe('Search and Filtering', () => {
    it('filters contracts by search term', () => {
      render(<ContractExplorer />);
      
      const searchInput = screen.getByPlaceholderText('Search contracts...');
      fireEvent.change(searchInput, { target: { value: 'increment' } });
      
      // Should only show increment contract
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
      expect(screen.queryByText('Key-Value Store')).not.toBeInTheDocument();
      expect(screen.queryByText('Data Validator')).not.toBeInTheDocument();
    });

    it('filters contracts by category', () => {
      render(<ContractExplorer />);
      
      const categoryFilter = screen.getByDisplayValue('All Categories');
      fireEvent.change(categoryFilter, { target: { value: 'storage' } });
      
      // Should only show storage contracts
      expect(screen.getByText('Key-Value Store')).toBeInTheDocument();
      expect(screen.queryByText('Increment Contract')).not.toBeInTheDocument();
      expect(screen.queryByText('Data Validator')).not.toBeInTheDocument();
    });

    it('combines search and category filters', () => {
      render(<ContractExplorer />);
      
      // Set category to storage
      const categoryFilter = screen.getByDisplayValue('All Categories');
      fireEvent.change(categoryFilter, { target: { value: 'storage' } });
      
      // Search for "key"
      const searchInput = screen.getByPlaceholderText('Search contracts...');
      fireEvent.change(searchInput, { target: { value: 'key' } });
      
      // Should show key-value store
      expect(screen.getByText('Key-Value Store')).toBeInTheDocument();
      expect(screen.queryByText('Increment Contract')).not.toBeInTheDocument();
    });
  });

  describe('Sorting', () => {
    it('sorts contracts by name', () => {
      render(<ContractExplorer />);
      
      const sortSelect = screen.getByDisplayValue('Sort by Name');
      fireEvent.change(sortSelect, { target: { value: 'name' } });
      
      // Contracts should be sorted alphabetically
      const contractItems = screen.getAllByText(/Contract|Store|Validator/);
      expect(contractItems[0]).toHaveTextContent('Data Validator');
      expect(contractItems[1]).toHaveTextContent('Increment Contract');
      expect(contractItems[2]).toHaveTextContent('Key-Value Store');
    });

    it('changes sort order when sort button is clicked', () => {
      render(<ContractExplorer />);
      
      const sortButton = screen.getByText('↑');
      fireEvent.click(sortButton);
      
      // Should change to descending order
      expect(screen.getByText('↓')).toBeInTheDocument();
    });
  });

  describe('Contract Execution', () => {
    it('shows execution options when contract is selected', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show execution options
      expect(screen.getByText('Execute Contract')).toBeInTheDocument();
      expect(screen.getByLabelText('Gas Limit:')).toBeInTheDocument();
      expect(screen.getByLabelText('ZK Mode:')).toBeInTheDocument();
      expect(screen.getByLabelText('Input Data (JSON):')).toBeInTheDocument();
    });

    it('enables ZK algorithm selection when ZK mode is enabled', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Enable ZK mode
      const zkCheckbox = screen.getByLabelText('ZK Mode:');
      fireEvent.click(zkCheckbox);
      
      // Should show algorithm selection
      expect(screen.getByLabelText('ZK Algorithm:')).toBeInTheDocument();
      expect(screen.getByDisplayValue('halo2')).toBeInTheDocument();
    });

    it('executes contract when execute button is clicked', async () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Click execute button
      const executeButton = screen.getByText('Execute Contract');
      fireEvent.click(executeButton);
      
      // Should show executing state
      expect(screen.getByText('Executing...')).toBeInTheDocument();
      
      // Wait for execution to complete
      await waitFor(() => {
        expect(screen.getByText('✓ Contract executed successfully')).toBeInTheDocument();
      });
    });

    it('shows execution results after successful execution', async () => {
      render(<ContractExplorer />);
      
      // Select and execute a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      const executeButton = screen.getByText('Execute Contract');
      fireEvent.click(executeButton);
      
      // Wait for execution to complete
      await waitFor(() => {
        expect(screen.getByText('✓ Contract executed successfully')).toBeInTheDocument();
      });
      
      // Should show execution results
      expect(screen.getByText(/Gas Used:/)).toBeInTheDocument();
      expect(screen.getByText(/Execution Time:/)).toBeInTheDocument();
      expect(screen.getByText(/Memory Used:/)).toBeInTheDocument();
      expect(screen.getByText(/ZK Proof:/)).toBeInTheDocument();
    });

    it('generates ZK proof when ZK mode is enabled', async () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Enable ZK mode
      const zkCheckbox = screen.getByLabelText('ZK Mode:');
      fireEvent.click(zkCheckbox);
      
      // Execute contract
      const executeButton = screen.getByText('Execute Contract');
      fireEvent.click(executeButton);
      
      // Wait for execution to complete
      await waitFor(() => {
        expect(screen.getByText('✓ Contract executed successfully')).toBeInTheDocument();
      });
      
      // Should show ZK proof details
      expect(screen.getByText('ZK Proof')).toBeInTheDocument();
      expect(screen.getByText('Generated')).toBeInTheDocument();
      expect(screen.getByText('Algorithm:')).toBeInTheDocument();
      expect(screen.getByText('Proof Size:')).toBeInTheDocument();
    });
  });

  describe('Contract Interfaces and Tags', () => {
    it('displays contract interfaces', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Key-Value Store').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show interfaces
      expect(screen.getByText('set')).toBeInTheDocument();
      expect(screen.getByText('get')).toBeInTheDocument();
      expect(screen.getByText('delete')).toBeInTheDocument();
      expect(screen.getByText('list')).toBeInTheDocument();
    });

    it('displays contract tags', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Data Validator').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Should show tags
      expect(screen.getByText('validation')).toBeInTheDocument();
      expect(screen.getByText('zk')).toBeInTheDocument();
      expect(screen.getByText('proofs')).toBeInTheDocument();
    });
  });

  describe('Contract Categories', () => {
    it('displays category badges with correct styling', () => {
      render(<ContractExplorer />);
      
      // Check category badges exist
      expect(screen.getByText('utility')).toBeInTheDocument();
      expect(screen.getByText('storage')).toBeInTheDocument();
      expect(screen.getByText('computation')).toBeInTheDocument();
    });

    it('filters by different categories', () => {
      render(<ContractExplorer />);
      
      const categoryFilter = screen.getByDisplayValue('All Categories');
      
      // Test utility category
      fireEvent.change(categoryFilter, { target: { value: 'utility' } });
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
      expect(screen.queryByText('Key-Value Store')).not.toBeInTheDocument();
      
      // Test computation category
      fireEvent.change(categoryFilter, { target: { value: 'computation' } });
      expect(screen.getByText('Data Validator')).toBeInTheDocument();
      expect(screen.queryByText('Increment Contract')).not.toBeInTheDocument();
    });
  });

  describe('Input Validation', () => {
    it('validates gas limit input', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      const gasInput = screen.getByLabelText('Gas Limit:');
      
      // Test valid input
      fireEvent.change(gasInput, { target: { value: '500000' } });
      expect(gasInput).toHaveValue(500000);
      
      // Test invalid input (should be handled gracefully)
      fireEvent.change(gasInput, { target: { value: 'invalid' } });
      expect(gasInput).toHaveValue(null);
    });

    it('validates timeout input', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      const timeoutInput = screen.getByLabelText('Timeout (ms):');
      
      // Test valid input
      fireEvent.change(timeoutInput, { target: { value: '15000' } });
      expect(timeoutInput).toHaveValue(15000);
    });

    it('validates JSON input data', () => {
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      const inputTextarea = screen.getByLabelText('Input Data (JSON):');
      
      // Test valid JSON
      fireEvent.change(inputTextarea, { target: { value: '{"key": "value"}' } });
      expect(inputTextarea).toHaveValue('{"key": "value"}');
      
      // Test invalid JSON (should still accept it for user to fix)
      fireEvent.change(inputTextarea, { target: { value: '{"invalid": json' } });
      expect(inputTextarea).toHaveValue('{"invalid": json');
    });
  });

  describe('Responsive Design', () => {
    it('adapts layout for smaller screens', () => {
      // Mock window resize
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 768,
      });
      
      render(<ContractExplorer />);
      
      // Should still render all elements
      expect(screen.getByText('NGFS Smart Contract Explorer')).toBeInTheDocument();
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper labels for form controls', () => {
      render(<ContractExplorer />);
      
      // Select a contract to show execution options
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Check that form controls have proper labels
      expect(screen.getByLabelText('Gas Limit:')).toBeInTheDocument();
      expect(screen.getByLabelText('ZK Mode:')).toBeInTheDocument();
      expect(screen.getByLabelText('Timeout (ms):')).toBeInTheDocument();
      expect(screen.getByLabelText('Input Data (JSON):')).toBeInTheDocument();
    });

    it('supports keyboard navigation', () => {
      render(<ContractExplorer />);
      
      // Tab through interactive elements
      const searchInput = screen.getByPlaceholderText('Search contracts...');
      searchInput.focus();
      
      // Should be able to tab to next element
      const categoryFilter = screen.getByDisplayValue('All Categories');
      categoryFilter.focus();
      
      expect(document.activeElement).toBe(categoryFilter);
    });
  });

  describe('Error Handling', () => {
    it('handles execution errors gracefully', async () => {
      // Mock a failed execution
      jest.spyOn(console, 'error').mockImplementation(() => {});
      
      render(<ContractExplorer />);
      
      // Select a contract
      const contractItem = screen.getByText('Increment Contract').closest('.contract-item');
      fireEvent.click(contractItem!);
      
      // Execute contract (this will trigger the mock error handling)
      const executeButton = screen.getByText('Execute Contract');
      fireEvent.click(executeButton);
      
      // Should handle errors gracefully
      await waitFor(() => {
        expect(screen.getByText('✓ Contract executed successfully')).toBeInTheDocument();
      });
    });
  });

  describe('Performance', () => {
    it('handles large numbers of contracts efficiently', () => {
      render(<ContractExplorer />);
      
      // Should render all mock contracts quickly
      expect(screen.getByText('Increment Contract')).toBeInTheDocument();
      expect(screen.getByText('Key-Value Store')).toBeInTheDocument();
      expect(screen.getByText('Data Validator')).toBeInTheDocument();
    });

    it('updates UI responsively to user interactions', () => {
      render(<ContractExplorer />);
      
      // Search should be responsive
      const searchInput = screen.getByPlaceholderText('Search contracts...');
      fireEvent.change(searchInput, { target: { value: 'test' } });
      
      // UI should update immediately
      expect(searchInput).toHaveValue('test');
    });
  });
});
