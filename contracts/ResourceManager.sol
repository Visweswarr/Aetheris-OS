// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ResourceManager
 * @dev A contract for managing resource allocation in Aetheris OS
 * @author Aetheris OS Team
 */
contract ResourceManager {
    struct ResourceAllocation {
        uint256 cpuLimit;
        uint256 memoryLimit;
        uint256 storageLimit;
        uint256 networkLimit;
        uint256 allocatedAt;
        address allocatedBy;
        bool active;
    }
    
    struct ResourceUsage {
        uint256 cpuUsed;
        uint256 memoryUsed;
        uint256 storageUsed;
        uint256 networkUsed;
        uint256 lastUpdated;
    }
    
    mapping(address => ResourceAllocation) public allocations;
    mapping(address => ResourceUsage) public usage;
    mapping(address => bool) public authorizedAllocators;
    
    address public owner;
    uint256 public totalAllocatedCPU;
    uint256 public totalAllocatedMemory;
    uint256 public totalAllocatedStorage;
    uint256 public totalAllocatedNetwork;
    
    event ResourceAllocated(
        address indexed contractAddress,
        uint256 cpuLimit,
        uint256 memoryLimit,
        uint256 storageLimit,
        uint256 networkLimit,
        address indexed allocatedBy
    );
    
    event ResourceDeallocated(address indexed contractAddress, address indexed deallocatedBy);
    
    event ResourceUsageUpdated(
        address indexed contractAddress,
        uint256 cpuUsed,
        uint256 memoryUsed,
        uint256 storageUsed,
        uint256 networkUsed
    );
    
    event AllocatorAuthorized(address indexed allocator, address indexed authorizedBy);
    event AllocatorDeauthorized(address indexed allocator, address indexed deauthorizedBy);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }
    
    modifier onlyAuthorizedAllocator() {
        require(
            msg.sender == owner || authorizedAllocators[msg.sender],
            "Only authorized allocators can call this function"
        );
        _;
    }
    
    /**
     * @dev Constructor initializes the resource manager
     */
    constructor() {
        owner = msg.sender;
        authorizedAllocators[msg.sender] = true;
    }
    
    /**
     * @dev Allocate resources to a contract
     * @param _contractAddress Address of the contract
     * @param _cpuLimit CPU limit in milliseconds per second
     * @param _memoryLimit Memory limit in bytes
     * @param _storageLimit Storage limit in bytes
     * @param _networkLimit Network limit in bytes per second
     */
    function allocateResources(
        address _contractAddress,
        uint256 _cpuLimit,
        uint256 _memoryLimit,
        uint256 _storageLimit,
        uint256 _networkLimit
    ) public onlyAuthorizedAllocator {
        require(_contractAddress != address(0), "Contract address cannot be zero");
        require(_cpuLimit > 0, "CPU limit must be greater than zero");
        require(_memoryLimit > 0, "Memory limit must be greater than zero");
        require(_storageLimit > 0, "Storage limit must be greater than zero");
        require(_networkLimit > 0, "Network limit must be greater than zero");
        
        // Deallocate existing resources if any
        if (allocations[_contractAddress].active) {
            _deallocateResources(_contractAddress);
        }
        
        // Allocate new resources
        allocations[_contractAddress] = ResourceAllocation({
            cpuLimit: _cpuLimit,
            memoryLimit: _memoryLimit,
            storageLimit: _storageLimit,
            networkLimit: _networkLimit,
            allocatedAt: block.timestamp,
            allocatedBy: msg.sender,
            active: true
        });
        
        // Update totals
        totalAllocatedCPU += _cpuLimit;
        totalAllocatedMemory += _memoryLimit;
        totalAllocatedStorage += _storageLimit;
        totalAllocatedNetwork += _networkLimit;
        
        emit ResourceAllocated(
            _contractAddress,
            _cpuLimit,
            _memoryLimit,
            _storageLimit,
            _networkLimit,
            msg.sender
        );
    }
    
    /**
     * @dev Deallocate resources from a contract
     * @param _contractAddress Address of the contract
     */
    function deallocateResources(address _contractAddress) public onlyAuthorizedAllocator {
        require(allocations[_contractAddress].active, "No active allocation for this contract");
        
        _deallocateResources(_contractAddress);
        
        emit ResourceDeallocated(_contractAddress, msg.sender);
    }
    
    /**
     * @dev Internal function to deallocate resources
     * @param _contractAddress Address of the contract
     */
    function _deallocateResources(address _contractAddress) internal {
        ResourceAllocation storage allocation = allocations[_contractAddress];
        
        // Update totals
        totalAllocatedCPU -= allocation.cpuLimit;
        totalAllocatedMemory -= allocation.memoryLimit;
        totalAllocatedStorage -= allocation.storageLimit;
        totalAllocatedNetwork -= allocation.networkLimit;
        
        // Deactivate allocation
        allocation.active = false;
    }
    
    /**
     * @dev Update resource usage for a contract
     * @param _contractAddress Address of the contract
     * @param _cpuUsed CPU used in milliseconds
     * @param _memoryUsed Memory used in bytes
     * @param _storageUsed Storage used in bytes
     * @param _networkUsed Network used in bytes
     */
    function updateResourceUsage(
        address _contractAddress,
        uint256 _cpuUsed,
        uint256 _memoryUsed,
        uint256 _storageUsed,
        uint256 _networkUsed
    ) public onlyAuthorizedAllocator {
        require(allocations[_contractAddress].active, "No active allocation for this contract");
        
        ResourceAllocation storage allocation = allocations[_contractAddress];
        ResourceUsage storage currentUsage = usage[_contractAddress];
        
        // Check if usage exceeds limits
        require(_cpuUsed <= allocation.cpuLimit, "CPU usage exceeds limit");
        require(_memoryUsed <= allocation.memoryLimit, "Memory usage exceeds limit");
        require(_storageUsed <= allocation.storageLimit, "Storage usage exceeds limit");
        require(_networkUsed <= allocation.networkLimit, "Network usage exceeds limit");
        
        // Update usage
        currentUsage.cpuUsed = _cpuUsed;
        currentUsage.memoryUsed = _memoryUsed;
        currentUsage.storageUsed = _storageUsed;
        currentUsage.networkUsed = _networkUsed;
        currentUsage.lastUpdated = block.timestamp;
        
        emit ResourceUsageUpdated(_contractAddress, _cpuUsed, _memoryUsed, _storageUsed, _networkUsed);
    }
    
    /**
     * @dev Authorize an allocator
     * @param _allocator Address of the allocator to authorize
     */
    function authorizeAllocator(address _allocator) public onlyOwner {
        require(_allocator != address(0), "Allocator address cannot be zero");
        require(!authorizedAllocators[_allocator], "Allocator is already authorized");
        
        authorizedAllocators[_allocator] = true;
        emit AllocatorAuthorized(_allocator, msg.sender);
    }
    
    /**
     * @dev Deauthorize an allocator
     * @param _allocator Address of the allocator to deauthorize
     */
    function deauthorizeAllocator(address _allocator) public onlyOwner {
        require(_allocator != address(0), "Allocator address cannot be zero");
        require(authorizedAllocators[_allocator], "Allocator is not authorized");
        require(_allocator != owner, "Cannot deauthorize owner");
        
        authorizedAllocators[_allocator] = false;
        emit AllocatorDeauthorized(_allocator, msg.sender);
    }
    
    /**
     * @dev Get resource allocation for a contract
     * @param _contractAddress Address of the contract
     * @return allocation Resource allocation details
     */
    function getResourceAllocation(address _contractAddress) public view returns (ResourceAllocation memory allocation) {
        return allocations[_contractAddress];
    }
    
    /**
     * @dev Get resource usage for a contract
     * @param _contractAddress Address of the contract
     * @return usage Resource usage details
     */
    function getResourceUsage(address _contractAddress) public view returns (ResourceUsage memory usage) {
        return usage[_contractAddress];
    }
    
    /**
     * @dev Get total resource allocation
     * @return totalCPU Total allocated CPU
     * @return totalMemory Total allocated memory
     * @return totalStorage Total allocated storage
     * @return totalNetwork Total allocated network
     */
    function getTotalResourceAllocation() public view returns (
        uint256 totalCPU,
        uint256 totalMemory,
        uint256 totalStorage,
        uint256 totalNetwork
    ) {
        return (totalAllocatedCPU, totalAllocatedMemory, totalAllocatedStorage, totalAllocatedNetwork);
    }
    
    /**
     * @dev Check if an address is an authorized allocator
     * @param _allocator Address to check
     * @return isAuthorized Whether the address is authorized
     */
    function isAuthorizedAllocator(address _allocator) public view returns (bool isAuthorized) {
        return _allocator == owner || authorizedAllocators[_allocator];
    }
    
    /**
     * @dev Get contract information
     * @return contractName Name of the contract
     * @return version Version of the contract
     * @return contractOwner Contract owner
     * @return totalAllocations Total number of active allocations
     */
    function getContractInfo() public view returns (
        string memory contractName,
        string memory version,
        address contractOwner,
        uint256 totalAllocations
    ) {
        // Count active allocations
        uint256 activeAllocations = 0;
        // Note: In a real implementation, you would iterate through all allocations
        // This is a simplified version for demonstration
        
        return ("ResourceManager", "1.0.0", owner, activeAllocations);
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
