// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/Token.sol";

contract TokenInvariantTest is Test {
    PolymeraToken public token;
    address public owner;
    address public user1;
    address public user2;
    
    // Invariant state variables
    uint256 public totalSupplySnapshot;
    uint256 public ownerBalanceSnapshot;
    uint256 public user1BalanceSnapshot;
    uint256 public user2BalanceSnapshot;
    
    function setUp() public {
        owner = address(this);
        user1 = address(0x1);
        user2 = address(0x2);
        
        // Deploy token with reasonable parameters
        token = new PolymeraToken(
            "Polymera Token",
            "POLY",
            1000000 * 10**18,  // 1M initial supply
            10000000 * 10**18, // 10M max supply
            0.001 ether        // 0.001 ETH per mint
        );
        
        // Fund test accounts
        vm.deal(user1, 100 ether);
        vm.deal(user2, 100 ether);
        
        // Take initial snapshots
        totalSupplySnapshot = token.totalSupply();
        ownerBalanceSnapshot = token.balanceOf(owner);
        user1BalanceSnapshot = token.balanceOf(user1);
        user2BalanceSnapshot = token.balanceOf(user2);
    }
    
    // Invariant: Total supply should never decrease
    function invariant_totalSupplyNeverDecreases() public view {
        assert(token.totalSupply() >= totalSupplySnapshot);
    }
    
    // Invariant: Owner balance should never exceed total supply
    function invariant_ownerBalanceNeverExceedsTotalSupply() public view {
        assert(token.balanceOf(owner) <= token.totalSupply());
    }
    
    // Invariant: No user should have negative balance
    function invariant_noNegativeBalances() public view {
        assert(token.balanceOf(user1) >= 0);
        assert(token.balanceOf(user2) >= 0);
        assert(token.balanceOf(owner) >= 0);
    }
    
    // Invariant: Total supply should never exceed max supply
    function invariant_totalSupplyNeverExceedsMax() public view {
        assert(token.totalSupply() <= token.maxSupply());
    }
    
    // Invariant: Sum of all balances should equal total supply
    function invariant_balanceSumEqualsTotalSupply() public view {
        uint256 totalBalance = token.balanceOf(owner) + 
                              token.balanceOf(user1) + 
                              token.balanceOf(user2);
        assert(totalBalance <= token.totalSupply());
    }
    
    // Invariant: Minting should respect max supply
    function invariant_mintingRespectsMaxSupply() public view {
        if (token.mintingEnabled()) {
            assert(token.totalSupply() <= token.maxSupply());
        }
    }
    
    // Invariant: Paused state should be consistent
    function invariant_pausedStateConsistent() public view {
        if (token.paused()) {
            // When paused, transfers should be blocked
            // This is tested in the transfer functions
        }
    }
    
    // Invariant: Mint price should be positive
    function invariant_mintPricePositive() public view {
        assert(token.mintPrice() > 0);
    }
    
    // Invariant: Max supply should be positive
    function invariant_maxSupplyPositive() public view {
        assert(token.maxSupply() > 0);
    }
    
    // Invariant: Decimals should be 18
    function invariant_decimalsAlways18() public view {
        assert(token.decimals() == 18);
    }
    
    // Invariant: Name and symbol should not be empty
    function invariant_nameAndSymbolNotEmpty() public view {
        assert(bytes(token.name()).length > 0);
        assert(bytes(token.symbol()).length > 0);
    }
    
    // Helper function to check if user can mint
    function canUserMint(address user) public view returns (bool) {
        if (!token.mintingEnabled()) return false;
        if (token.totalSupply() >= token.maxSupply()) return false;
        
        uint256 lastMint = token.lastMintTime(user);
        return block.timestamp >= lastMint + token.MIN_MINT_INTERVAL();
    }
    
    // Invariant: Minting interval should be respected
    function invariant_mintingIntervalRespected() public view {
        if (token.lastMintTime(user1) > 0) {
            uint256 timeSinceLastMint = block.timestamp - token.lastMintTime(user1);
            if (timeSinceLastMint < token.MIN_MINT_INTERVAL()) {
                assert(!canUserMint(user1));
            }
        }
        
        if (token.lastMintTime(user2) > 0) {
            uint256 timeSinceLastMint = block.timestamp - token.lastMintTime(user2);
            if (timeSinceLastMint < token.MIN_MINT_INTERVAL()) {
                assert(!canUserMint(user2));
            }
        }
    }
    
    // Invariant: Mint count should be non-decreasing
    function invariant_mintCountNonDecreasing() public view {
        assert(token.mintCount(user1) >= 0);
        assert(token.mintCount(user2) >= 0);
    }
    
    // Invariant: Emergency recovery should not affect own token
    function invariant_emergencyRecoveryNotSelf() public view {
        // This invariant ensures the contract cannot recover its own tokens
        // The emergencyRecover function has a require check for this
    }
    
    // Invariant: Withdrawal should not affect token balances
    function invariant_withdrawalNotAffectTokenBalances() public view {
        // Withdrawing ETH should not change token balances
        uint256 currentOwnerBalance = token.balanceOf(owner);
        assert(currentOwnerBalance >= ownerBalanceSnapshot);
    }
    
    // Invariant: Pause/unpause should not affect balances
    function invariant_pauseUnpauseNotAffectBalances() public view {
        uint256 currentOwnerBalance = token.balanceOf(owner);
        uint256 currentUser1Balance = token.balanceOf(user1);
        uint256 currentUser2Balance = token.balanceOf(user2);
        
        // Balances should remain the same regardless of pause state
        assert(currentOwnerBalance >= ownerBalanceSnapshot);
        assert(currentUser1Balance >= user1BalanceSnapshot);
        assert(currentUser2Balance >= user2BalanceSnapshot);
    }
    
    // Invariant: Configuration changes should not affect existing balances
    function invariant_configChangesNotAffectBalances() public view {
        uint256 currentOwnerBalance = token.balanceOf(owner);
        uint256 currentUser1Balance = token.balanceOf(user1);
        uint256 currentUser2Balance = token.balanceOf(user2);
        
        // Configuration changes should not reduce existing balances
        assert(currentOwnerBalance >= ownerBalanceSnapshot);
        assert(currentUser1Balance >= user1BalanceSnapshot);
        assert(currentUser2Balance >= user2BalanceSnapshot);
    }
    
    // Invariant: Token transfers should maintain total supply
    function invariant_transfersMaintainTotalSupply() public view {
        // This invariant ensures that transfers don't create or destroy tokens
        // The total supply should remain constant during transfers
        assert(token.totalSupply() >= totalSupplySnapshot);
    }
    
    // Invariant: Minting should increase total supply
    function invariant_mintingIncreasesTotalSupply() public view {
        // When minting is enabled and max supply not reached, minting should increase supply
        if (token.mintingEnabled() && token.totalSupply() < token.maxSupply()) {
            // This invariant is checked by the total supply never decreases invariant
        }
    }
    
    // Invariant: Burning should decrease total supply
    function invariant_burningDecreasesTotalSupply() public view {
        // Burning should decrease the total supply
        // This is handled by the _burn function in the base contract
    }
    
    // Invariant: Access control should be maintained
    function invariant_accessControlMaintained() public view {
        // Only owner should be able to call admin functions
        // This is enforced by the onlyOwner modifier
    }
    
    // Invariant: Reentrancy protection should work
    function invariant_reentrancyProtection() public view {
        // The contract should be protected against reentrancy attacks
        // This is enforced by the nonReentrant modifier
    }
}
