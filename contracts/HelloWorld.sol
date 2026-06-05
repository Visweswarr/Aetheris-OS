// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title HelloWorld
 * @dev A simple hello world contract for testing Aetheris OS integration
 * @author Aetheris OS Team
 */
contract HelloWorld {
    string private greeting;
    address public owner;
    uint256 public callCount;
    mapping(address => uint256) public userCallCounts;
    
    event GreetingUpdated(string newGreeting, address updatedBy);
    event HelloCalled(address caller, string message, uint256 timestamp);
    
    /**
     * @dev Constructor sets the initial greeting and owner
     * @param _greeting The initial greeting message
     */
    constructor(string memory _greeting) {
        greeting = _greeting;
        owner = msg.sender;
        callCount = 0;
    }
    
    /**
     * @dev Get the current greeting
     * @return The current greeting string
     */
    function getGreeting() public view returns (string memory) {
        return greeting;
    }
    
    /**
     * @dev Set a new greeting (only owner)
     * @param _newGreeting The new greeting message
     */
    function setGreeting(string memory _newGreeting) public {
        require(msg.sender == owner, "Only owner can set greeting");
        greeting = _newGreeting;
        emit GreetingUpdated(_newGreeting, msg.sender);
    }
    
    /**
     * @dev Say hello with a custom message
     * @param _message The message to include with hello
     * @return The combined greeting and message
     */
    function sayHello(string memory _message) public returns (string memory) {
        callCount++;
        userCallCounts[msg.sender]++;
        
        string memory result = string(abi.encodePacked(greeting, ", ", _message, "!"));
        
        emit HelloCalled(msg.sender, _message, block.timestamp);
        
        return result;
    }
    
    /**
     * @dev Get call statistics
     * @return totalCalls Total number of calls
     * @return userCalls Number of calls by the current user
     */
    function getStats() public view returns (uint256 totalCalls, uint256 userCalls) {
        return (callCount, userCallCounts[msg.sender]);
    }
    
    /**
     * @dev Get contract information
     * @return contractName Name of the contract
     * @return version Version of the contract
     * @return contractOwner Address of the contract owner
     */
    function getInfo() public view returns (string memory contractName, string memory version, address contractOwner) {
        return ("HelloWorld", "1.0.0", owner);
    }
    
    /**
     * @dev Emergency function to transfer ownership
     * @param _newOwner Address of the new owner
     */
    function transferOwnership(address _newOwner) public {
        require(msg.sender == owner, "Only owner can transfer ownership");
        require(_newOwner != address(0), "New owner cannot be zero address");
        owner = _newOwner;
    }
}
