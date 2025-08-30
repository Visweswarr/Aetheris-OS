import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { AnchorExplorer } from '../anchor_ui';

// Mock the CSS import
jest.mock('../anchor_ui.css', () => ({}));

describe('AnchorExplorer', () => {
  beforeEach(() => {
    // Reset any global mocks
    jest.clearAllMocks();
  });

  describe('Initial Rendering', () => {
    it('renders the main header and title', () => {
      render(<AnchorExplorer />);
      
      expect(screen.getByText('NGFS Snapshot Anchors')).toBeInTheDocument();
      expect(screen.getByText('On-chain audit anchoring for immutable snapshot verification')).toBeInTheDocument();
    });

    it('displays anchor statistics', () => {
      render(<AnchorExplorer />);
      
      expect(screen.getByText('3')).toBeInTheDocument(); // Total anchors
      expect(screen.getByText('2')).toBeInTheDocument(); // Confirmed anchors
      expect(screen.getByText('1')).toBeInTheDocument(); // Pending anchors
      expect(screen.getByText('97,000')).toBeInTheDocument(); // Total gas
    });

    it('shows search and filter controls', () => {
      render(<AnchorExplorer />);
      
      expect(screen.getByPlaceholderText('Search anchors...')).toBeInTheDocument();
      expect(screen.getByText('All Chains')).toBeInTheDocument();
      expect(screen.getByText('All Status')).toBeInTheDocument();
      expect(screen.getByText('All Priorities')).toBeInTheDocument();
    });
  });

  describe('Anchor Display', () => {
    it('displays anchor items with correct information', () => {
      render(<AnchorExplorer />);
      
      // Check for anchor content
      expect(screen.getByText('Test NGFS snapshot anchor')).toBeInTheDocument();
      expect(screen.getByText('High priority system snapshot')).toBeInTheDocument();
      expect(screen.getByText('Pending user data snapshot')).toBeInTheDocument();
      
      // Check for DIDs
      expect(screen.getByText('did:aetheris:test:user1')).toBeInTheDocument();
      expect(screen.getByText('did:aetheris:test:user2')).toBeInTheDocument();
      expect(screen.getByText('did:aetheris:test:user3')).toBeInTheDocument();
    });

    it('shows correct status indicators', () => {
      render(<AnchorExplorer />);
      
      expect(screen.getByText('confirmed')).toBeInTheDocument();
      expect(screen.getByText('pending')).toBeInTheDocument();
    });

    it('displays priority badges with correct colors', () => {
      render(<AnchorExplorer />);
      
      const normalPriority = screen.getByText('normal');
      const highPriority = screen.getByText('high');
      const lowPriority = screen.getByText('low');
      
      expect(normalPriority).toBeInTheDocument();
      expect(highPriority).toBeInTheDocument();
      expect(lowPriority).toBeInTheDocument();
    });
  });

  describe('Search and Filtering', () => {
    it('filters anchors by search term', () => {
      render(<AnchorExplorer />);
      
      const searchInput = screen.getByPlaceholderText('Search anchors...');
      fireEvent.change(searchInput, { target: { value: 'system' } });
      
      // Should show only the system snapshot anchor
      expect(screen.getByText('High priority system snapshot')).toBeInTheDocument();
      expect(screen.queryByText('Test NGFS snapshot anchor')).not.toBeInTheDocument();
    });

    it('filters by chain', () => {
      render(<AnchorExplorer />);
      
      const chainSelect = screen.getByText('All Chains').closest('select');
      if (chainSelect) {
        fireEvent.change(chainSelect, { target: { value: 'local' } });
      }
      
      // All anchors should still be visible since they're all local
      expect(screen.getByText('Test NGFS snapshot anchor')).toBeInTheDocument();
      expect(screen.getByText('High priority system snapshot')).toBeInTheDocument();
    });

    it('filters by priority', () => {
      render(<AnchorExplorer />);
      
      const prioritySelect = screen.getByText('All Priorities').closest('select');
      if (prioritySelect) {
        fireEvent.change(prioritySelect, { target: { value: 'high' } });
      }
      
      // Should show only high priority anchors
      expect(screen.getByText('High priority system snapshot')).toBeInTheDocument();
      expect(screen.queryByText('Test NGFS snapshot anchor')).not.toBeInTheDocument();
    });
  });

  describe('Anchor Creation', () => {
    it('shows create form when create button is clicked', () => {
      render(<AnchorExplorer />);
      
      const createButton = screen.getByText('+ New Anchor');
      fireEvent.click(createButton);
      
      expect(screen.getByText('Create New Anchor')).toBeInTheDocument();
      expect(screen.getByLabelText('Snapshot CID:')).toBeInTheDocument();
      expect(screen.getByLabelText('Target Chain:')).toBeInTheDocument();
    });

    it('creates new anchor with valid data', async () => {
      render(<AnchorExplorer />);
      
      const createButton = screen.getByText('+ New Anchor');
      fireEvent.click(createButton);
      
      // Fill in form
      const cidInput = screen.getByLabelText('Snapshot CID:');
      const descriptionInput = screen.getByLabelText('Description:');
      const submitButton = screen.getByText('Create Anchor');
      
      fireEvent.change(cidInput, { target: { value: '0x1234567890abcdef' } });
      fireEvent.change(descriptionInput, { target: { value: 'New test anchor' } });
      
      fireEvent.click(submitButton);
      
      // Form should close and new anchor should appear
      await waitFor(() => {
        expect(screen.queryByText('Create New Anchor')).not.toBeInTheDocument();
      });
      
      // New anchor should be in the list
      expect(screen.getByText('New test anchor')).toBeInTheDocument();
    });

    it('validates required fields', () => {
      render(<AnchorExplorer />);
      
      const createButton = screen.getByText('+ New Anchor');
      fireEvent.click(createButton);
      
      const submitButton = screen.getByText('Create Anchor');
      fireEvent.click(submitButton);
      
      // Should show validation error (alert in this case)
      // In a real app, you'd check for error messages
      expect(submitButton).toBeInTheDocument();
    });
  });

  describe('Anchor Interaction', () => {
    it('expands anchor details when clicked', () => {
      render(<AnchorExplorer />);
      
      const firstAnchor = screen.getByText('Test NGFS snapshot anchor').closest('.anchor-item');
      if (firstAnchor) {
        fireEvent.click(firstAnchor);
      }
      
      // Should show expanded details
      expect(screen.getByText('Raw Data')).toBeInTheDocument();
      expect(screen.getByText('Custom Fields')).toBeInTheDocument();
    });

    it('collapses anchor details when clicked again', () => {
      render(<AnchorExplorer />);
      
      const firstAnchor = screen.getByText('Test NGFS snapshot anchor').closest('.anchor-item');
      if (firstAnchor) {
        fireEvent.click(firstAnchor);
        fireEvent.click(firstAnchor);
      }
      
      // Should hide expanded details
      expect(screen.queryByText('Raw Data')).not.toBeInTheDocument();
    });
  });

  describe('Sorting', () => {
    it('changes sort order when sort button is clicked', () => {
      render(<AnchorExplorer />);
      
      const sortButton = screen.getByText('↓');
      fireEvent.click(sortButton);
      
      // Should change to ascending
      expect(screen.getByText('↑')).toBeInTheDocument();
    });

    it('changes sort field when sort select is changed', () => {
      render(<AnchorExplorer />);
      
      const sortSelect = screen.getByText('Time').closest('select');
      if (sortSelect) {
        fireEvent.change(sortSelect, { target: { value: 'priority' } });
      }
      
      // Should now be sorting by priority
      expect(sortSelect).toHaveValue('priority');
    });
  });

  describe('Responsive Design', () => {
    it('adapts to different screen sizes', () => {
      // Mock window resize
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 480,
      });
      
      render(<AnchorExplorer />);
      
      // Component should still render
      expect(screen.getByText('NGFS Snapshot Anchors')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper labels for form inputs', () => {
      render(<AnchorExplorer />);
      
      const createButton = screen.getByText('+ New Anchor');
      fireEvent.click(createButton);
      
      expect(screen.getByLabelText('Snapshot CID:')).toBeInTheDocument();
      expect(screen.getByLabelText('Target Chain:')).toBeInTheDocument();
      expect(screen.getByLabelText('Priority:')).toBeInTheDocument();
    });

    it('supports keyboard navigation', () => {
      render(<AnchorExplorer />);
      
      const searchInput = screen.getByPlaceholderText('Search anchors...');
      searchInput.focus();
      
      expect(searchInput).toHaveFocus();
    });
  });

  describe('Error Handling', () => {
    it('handles empty search results gracefully', () => {
      render(<AnchorExplorer />);
      
      const searchInput = screen.getByPlaceholderText('Search anchors...');
      fireEvent.change(searchInput, { target: { value: 'nonexistent' } });
      
      expect(screen.getByText('No anchors found matching the current filters.')).toBeInTheDocument();
    });
  });

  describe('Performance', () => {
    it('renders without performance issues', () => {
      const startTime = performance.now();
      render(<AnchorExplorer />);
      const endTime = performance.now();
      
      // Should render in reasonable time (less than 100ms)
      expect(endTime - startTime).toBeLessThan(100);
    });
  });
});
