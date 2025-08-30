// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title PolymeraToken
 * @dev ERC20 token with advanced features for testing
 */
contract PolymeraToken is ERC20, Pausable, Ownable, ReentrancyGuard {
    // Events
    event Minted(address indexed to, uint256 amount);
    event Burned(address indexed from, uint256 amount);
    event Paused(address indexed account);
    event Unpaused(address indexed account);
    
    // State variables
    uint256 public maxSupply;
    uint256 public mintPrice;
    bool public mintingEnabled;
    
    // Mappings
    mapping(address => uint256) public lastMintTime;
    mapping(address => uint256) public mintCount;
    
    // Constants
    uint256 public constant MIN_MINT_INTERVAL = 1 hours;
    uint256 public constant MAX_MINT_PER_TX = 1000 * 10**18;
    
    // Modifiers
    modifier whenMintingEnabled() {
        require(mintingEnabled, "Minting is disabled");
        _;
    }
    
    modifier validMintAmount(uint256 amount) {
        require(amount > 0, "Amount must be greater than 0");
        require(amount <= MAX_MINT_PER_TX, "Amount exceeds max per transaction");
        require(totalSupply() + amount <= maxSupply, "Would exceed max supply");
        _;
    }
    
    modifier validMintInterval() {
        require(
            block.timestamp >= lastMintTime[msg.sender] + MIN_MINT_INTERVAL,
            "Must wait before next mint"
        );
        _;
    }
    
    /**
     * @dev Constructor
     * @param name Token name
     * @param symbol Token symbol
     * @param initialSupply Initial token supply
     * @param _maxSupply Maximum token supply
     * @param _mintPrice Price per mint
     */
    constructor(
        string memory name,
        string memory symbol,
        uint256 initialSupply,
        uint256 _maxSupply,
        uint256 _mintPrice
    ) ERC20(name, symbol) {
        require(_maxSupply > initialSupply, "Max supply must be greater than initial");
        require(_mintPrice > 0, "Mint price must be greater than 0");
        
        maxSupply = _maxSupply;
        mintPrice = _mintPrice;
        mintingEnabled = true;
        
        _mint(msg.sender, initialSupply);
    }
    
    /**
     * @dev Mint tokens (public)
     * @param amount Amount to mint
     */
    function mint(uint256 amount) 
        external 
        payable 
        whenMintingEnabled 
        validMintAmount(amount) 
        validMintInterval 
        nonReentrant 
    {
        require(msg.value >= mintPrice * amount, "Insufficient payment");
        
        lastMintTime[msg.sender] = block.timestamp;
        mintCount[msg.sender]++;
        
        _mint(msg.sender, amount);
        emit Minted(msg.sender, amount);
    }
    
    /**
     * @dev Mint tokens (owner only)
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function mintTo(address to, uint256 amount) 
        external 
        onlyOwner 
        validMintAmount(amount) 
    {
        _mint(to, amount);
        emit Minted(to, amount);
    }
    
    /**
     * @dev Burn tokens
     * @param amount Amount to burn
     */
    function burn(uint256 amount) external {
        require(amount > 0, "Amount must be greater than 0");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");
        
        _burn(msg.sender, amount);
        emit Burned(msg.sender, amount);
    }
    
    /**
     * @dev Pause all token transfers
     */
    function pause() external onlyOwner {
        _pause();
        emit Paused(msg.sender);
    }
    
    /**
     * @dev Unpause all token transfers
     */
    function unpause() external onlyOwner {
        _unpause();
        emit Unpaused(msg.sender);
    }
    
    /**
     * @dev Set minting enabled/disabled
     * @param enabled Whether minting is enabled
     */
    function setMintingEnabled(bool enabled) external onlyOwner {
        mintingEnabled = enabled;
    }
    
    /**
     * @dev Set mint price
     * @param newPrice New mint price
     */
    function setMintPrice(uint256 newPrice) external onlyOwner {
        require(newPrice > 0, "Price must be greater than 0");
        mintPrice = newPrice;
    }
    
    /**
     * @dev Set max supply
     * @param newMaxSupply New max supply
     */
    function setMaxSupply(uint256 newMaxSupply) external onlyOwner {
        require(newMaxSupply >= totalSupply(), "Max supply cannot be less than current supply");
        maxSupply = newMaxSupply;
    }
    
    /**
     * @dev Withdraw contract balance
     */
    function withdraw() external onlyOwner nonReentrant {
        uint256 balance = address(this).balance;
        require(balance > 0, "No balance to withdraw");
        
        (bool success, ) = payable(owner()).call{value: balance}("");
        require(success, "Withdrawal failed");
    }
    
    /**
     * @dev Get mint statistics for an address
     * @param account Address to check
     * @return lastMint Last mint timestamp
     * @return totalMints Total number of mints
     * @return canMint Whether address can mint now
     */
    function getMintStats(address account) external view returns (
        uint256 lastMint,
        uint256 totalMints,
        bool canMint
    ) {
        lastMint = lastMintTime[account];
        totalMints = mintCount[account];
        canMint = mintingEnabled && 
                   block.timestamp >= lastMint + MIN_MINT_INTERVAL &&
                   totalSupply() < maxSupply;
    }
    
    /**
     * @dev Override _beforeTokenTransfer to implement pausing
     */
    function _beforeTokenTransfer(
        address from,
        address to,
        uint256 amount
    ) internal virtual override {
        super._beforeTokenTransfer(from, to, amount);
        require(!paused(), "Token transfer while paused");
    }
    
    /**
     * @dev Override _afterTokenTransfer for additional logic
     */
    function _afterTokenTransfer(
        address from,
        address to,
        uint256 amount
    ) internal virtual override {
        super._afterTokenTransfer(from, to, amount);
        // Additional logic can be added here
    }
    
    /**
     * @dev Emergency function to recover stuck tokens
     * @param tokenAddress Address of stuck token
     * @param to Recipient address
     * @param amount Amount to recover
     */
    function emergencyRecover(
        address tokenAddress,
        address to,
        uint256 amount
    ) external onlyOwner {
        require(tokenAddress != address(this), "Cannot recover own token");
        require(to != address(0), "Invalid recipient");
        
        IERC20(tokenAddress).transfer(to, amount);
    }
    
    /**
     * @dev Get contract information
     */
    function getContractInfo() external view returns (
        string memory name_,
        string memory symbol_,
        uint256 decimals_,
        uint256 totalSupply_,
        uint256 maxSupply_,
        uint256 mintPrice_,
        bool mintingEnabled_,
        bool paused_
    ) {
        name_ = name();
        symbol_ = symbol();
        decimals_ = decimals();
        totalSupply_ = totalSupply();
        maxSupply_ = maxSupply;
        mintPrice_ = mintPrice;
        mintingEnabled_ = mintingEnabled;
        paused_ = paused();
    }
}

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
}
