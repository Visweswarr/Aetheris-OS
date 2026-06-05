// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TokenContract
 * @dev A simple ERC20-like token contract for testing Aetheris OS integration
 * @author Aetheris OS Team
 */
contract TokenContract {
    string public name;
    string public symbol;
    uint8 public decimals;
    uint256 public totalSupply;
    
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    address public owner;
    bool public paused;
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Mint(address indexed to, uint256 value);
    event Burn(address indexed from, uint256 value);
    event Pause();
    event Unpause();
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }
    
    /**
     * @dev Constructor initializes the token
     * @param _name Token name
     * @param _symbol Token symbol
     * @param _decimals Token decimals
     * @param _initialSupply Initial token supply
     */
    constructor(
        string memory _name,
        string memory _symbol,
        uint8 _decimals,
        uint256 _initialSupply
    ) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        totalSupply = _initialSupply * 10**_decimals;
        balanceOf[msg.sender] = totalSupply;
        owner = msg.sender;
        paused = false;
        
        emit Transfer(address(0), msg.sender, totalSupply);
    }
    
    /**
     * @dev Transfer tokens to another address
     * @param _to Address to transfer to
     * @param _value Amount to transfer
     * @return success Whether the transfer was successful
     */
    function transfer(address _to, uint256 _value) public whenNotPaused returns (bool success) {
        require(_to != address(0), "Cannot transfer to zero address");
        require(balanceOf[msg.sender] >= _value, "Insufficient balance");
        
        balanceOf[msg.sender] -= _value;
        balanceOf[_to] += _value;
        
        emit Transfer(msg.sender, _to, _value);
        return true;
    }
    
    /**
     * @dev Transfer tokens from one address to another
     * @param _from Address to transfer from
     * @param _to Address to transfer to
     * @param _value Amount to transfer
     * @return success Whether the transfer was successful
     */
    function transferFrom(address _from, address _to, uint256 _value) public whenNotPaused returns (bool success) {
        require(_to != address(0), "Cannot transfer to zero address");
        require(balanceOf[_from] >= _value, "Insufficient balance");
        require(allowance[_from][msg.sender] >= _value, "Insufficient allowance");
        
        balanceOf[_from] -= _value;
        balanceOf[_to] += _value;
        allowance[_from][msg.sender] -= _value;
        
        emit Transfer(_from, _to, _value);
        return true;
    }
    
    /**
     * @dev Approve spender to transfer tokens
     * @param _spender Address to approve
     * @param _value Amount to approve
     * @return success Whether the approval was successful
     */
    function approve(address _spender, uint256 _value) public whenNotPaused returns (bool success) {
        allowance[msg.sender][_spender] = _value;
        emit Approval(msg.sender, _spender, _value);
        return true;
    }
    
    /**
     * @dev Mint new tokens (only owner)
     * @param _to Address to mint tokens to
     * @param _value Amount to mint
     */
    function mint(address _to, uint256 _value) public onlyOwner whenNotPaused {
        require(_to != address(0), "Cannot mint to zero address");
        
        totalSupply += _value;
        balanceOf[_to] += _value;
        
        emit Mint(_to, _value);
        emit Transfer(address(0), _to, _value);
    }
    
    /**
     * @dev Burn tokens
     * @param _value Amount to burn
     */
    function burn(uint256 _value) public whenNotPaused {
        require(balanceOf[msg.sender] >= _value, "Insufficient balance to burn");
        
        balanceOf[msg.sender] -= _value;
        totalSupply -= _value;
        
        emit Burn(msg.sender, _value);
        emit Transfer(msg.sender, address(0), _value);
    }
    
    /**
     * @dev Pause the contract (only owner)
     */
    function pause() public onlyOwner {
        paused = true;
        emit Pause();
    }
    
    /**
     * @dev Unpause the contract (only owner)
     */
    function unpause() public onlyOwner {
        paused = false;
        emit Unpause();
    }
    
    /**
     * @dev Get token information
     * @return tokenName Token name
     * @return tokenSymbol Token symbol
     * @return tokenDecimals Token decimals
     * @return tokenTotalSupply Total token supply
     * @return tokenOwner Contract owner
     */
    function getTokenInfo() public view returns (
        string memory tokenName,
        string memory tokenSymbol,
        uint8 tokenDecimals,
        uint256 tokenTotalSupply,
        address tokenOwner
    ) {
        return (name, symbol, decimals, totalSupply, owner);
    }
    
    /**
     * @dev Get account information
     * @param _account Account address
     * @return accountBalance Account balance
     * @return accountAllowance Account allowance for msg.sender
     */
    function getAccountInfo(address _account) public view returns (uint256 accountBalance, uint256 accountAllowance) {
        return (balanceOf[_account], allowance[_account][msg.sender]);
    }
    
    /**
     * @dev Emergency function to transfer ownership
     * @param _newOwner Address of the new owner
     */
    function transferOwnership(address _newOwner) public onlyOwner {
        require(_newOwner != address(0), "New owner cannot be zero address");
        owner = _newOwner;
    }
}
