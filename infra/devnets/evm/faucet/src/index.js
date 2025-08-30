const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const morgan = require('morgan');
const { ethers } = require('ethers');
const { RateLimiterMemory } = require('rate-limiter-flexible');
const Joi = require('joi');
const winston = require('winston');

// Load environment variables
require('dotenv').config();

// Configure logging
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'polymera-faucet' },
  transports: [
    new winston.transports.File({ filename: 'error.log', level: 'error' }),
    new winston.transports.File({ filename: 'combined.log' }),
    new winston.transports.Console({
      format: winston.format.simple()
    })
  ]
});

// Initialize Express app
const app = express();
const PORT = process.env.PORT || 3000;

// Rate limiting
const rateLimiter = new RateLimiterMemory({
  keyGenerator: (req) => req.ip,
  points: 5, // 5 requests
  duration: 60, // per minute
});

// Middleware
app.use(helmet());
app.use(cors());
app.use(morgan('combined'));
app.use(express.json({ limit: '10mb' }));

// Ethereum provider setup
const rpcUrl = process.env.RPC_URL || 'http://localhost:8545';
const privateKey = process.env.PRIVATE_KEY || '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
const chainId = parseInt(process.env.CHAIN_ID || '31337');

let provider, wallet, faucetAddress;

// Initialize Ethereum connection
async function initializeEthereum() {
  try {
    provider = new ethers.JsonRpcProvider(rpcUrl);
    wallet = new ethers.Wallet(privateKey, provider);
    faucetAddress = await wallet.getAddress();
    
    const balance = await provider.getBalance(faucetAddress);
    const network = await provider.getNetwork();
    
    logger.info('Ethereum connection initialized', {
      faucetAddress,
      balance: ethers.formatEther(balance),
      chainId: network.chainId,
      rpcUrl
    });
    
    return true;
  } catch (error) {
    logger.error('Failed to initialize Ethereum connection', { error: error.message });
    return false;
  }
}

// Validation schemas
const fundRequestSchema = Joi.object({
  address: Joi.string().pattern(/^0x[a-fA-F0-9]{40}$/).required(),
  amount: Joi.string().pattern(/^\d+(\.\d+)?$/).optional().default('1.0'),
  token: Joi.string().valid('ETH', 'USDC', 'DAI').optional().default('ETH')
});

// Health check endpoint
app.get('/health', async (req, res) => {
  try {
    const ethereumStatus = await initializeEthereum();
    const balance = ethereumStatus ? await provider.getBalance(faucetAddress) : '0';
    
    res.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      ethereum: {
        connected: ethereumStatus,
        faucetAddress: faucetAddress || 'unknown',
        balance: ethereumStatus ? ethers.formatEther(balance) : 'unknown',
        chainId,
        rpcUrl
      },
      uptime: process.uptime()
    });
  } catch (error) {
    logger.error('Health check failed', { error: error.message });
    res.status(500).json({
      status: 'unhealthy',
      error: error.message,
      timestamp: new Date().toISOString()
    });
  }
});

// Fund endpoint
app.post('/fund', async (req, res) => {
  try {
    // Rate limiting
    await rateLimiter.consume(req.ip);
    
    // Validate request
    const { error, value } = fundRequestSchema.validate(req.body);
    if (error) {
      return res.status(400).json({
        error: 'Validation failed',
        details: error.details
      });
    }
    
    const { address, amount, token } = value;
    
    // Check if faucet has sufficient balance
    const faucetBalance = await provider.getBalance(faucetAddress);
    const requestAmount = ethers.parseEther(amount);
    
    if (faucetBalance < requestAmount) {
      logger.warn('Insufficient faucet balance', {
        requested: amount,
        available: ethers.formatEther(faucetBalance),
        address
      });
      
      return res.status(503).json({
        error: 'Insufficient faucet balance',
        requested: amount,
        available: ethers.formatEther(faucetBalance)
      });
    }
    
    // Send transaction
    const tx = await wallet.sendTransaction({
      to: address,
      value: requestAmount,
      gasLimit: 21000
    });
    
    logger.info('Funds sent successfully', {
      to: address,
      amount,
      token,
      txHash: tx.hash,
      faucetAddress
    });
    
    // Wait for confirmation
    const receipt = await tx.wait();
    
    res.json({
      success: true,
      message: `Successfully sent ${amount} ${token}`,
      transaction: {
        hash: tx.hash,
        blockNumber: receipt.blockNumber,
        gasUsed: receipt.gasUsed.toString(),
        effectiveGasPrice: receipt.effectiveGasPrice.toString()
      },
      recipient: address,
      amount,
      token
    });
    
  } catch (error) {
    if (error.name === 'RateLimiterError') {
      return res.status(429).json({
        error: 'Rate limit exceeded',
        message: 'Too many requests. Please try again later.'
      });
    }
    
    logger.error('Fund request failed', {
      error: error.message,
      body: req.body,
      ip: req.ip
    });
    
    res.status(500).json({
      error: 'Failed to send funds',
      message: error.message
    });
  }
});

// Get faucet info
app.get('/info', async (req, res) => {
  try {
    const balance = await provider.getBalance(faucetAddress);
    const nonce = await provider.getTransactionCount(faucetAddress);
    
    res.json({
      faucetAddress,
      balance: ethers.formatEther(balance),
      nonce,
      chainId,
      network: 'Local Devnet',
      supportedTokens: ['ETH', 'USDC', 'DAI'],
      rateLimit: {
        requests: 5,
        window: '60 seconds'
      }
    });
  } catch (error) {
    logger.error('Failed to get faucet info', { error: error.message });
    res.status(500).json({
      error: 'Failed to get faucet info',
      message: error.message
    });
  }
});

// Get transaction history
app.get('/transactions', async (req, res) => {
  try {
    const page = parseInt(req.query.page) || 1;
    const limit = Math.min(parseInt(req.query.limit) || 10, 100);
    
    // Get recent transactions from the faucet address
    const blockNumber = await provider.getBlockNumber();
    const transactions = [];
    
    // Look at recent blocks for faucet transactions
    for (let i = 0; i < Math.min(limit * 2, 100); i++) {
      const block = await provider.getBlock(blockNumber - i, true);
      if (block && block.transactions) {
        for (const tx of block.transactions) {
          if (tx.from === faucetAddress) {
            transactions.push({
              hash: tx.hash,
              blockNumber: block.number,
              timestamp: block.timestamp,
              to: tx.to,
              value: ethers.formatEther(tx.value),
              gasUsed: tx.gasLimit.toString()
            });
            
            if (transactions.length >= limit) break;
          }
        }
      }
      if (transactions.length >= limit) break;
    }
    
    res.json({
      transactions,
      pagination: {
        page,
        limit,
        total: transactions.length
      }
    });
  } catch (error) {
    logger.error('Failed to get transaction history', { error: error.message });
    res.status(500).json({
      error: 'Failed to get transaction history',
      message: error.message
    });
  }
});

// Error handling middleware
app.use((error, req, res, next) => {
  logger.error('Unhandled error', { error: error.message, stack: error.stack });
  res.status(500).json({
    error: 'Internal server error',
    message: 'Something went wrong'
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: 'Not found',
    message: 'The requested endpoint does not exist'
  });
});

// Start server
async function startServer() {
  try {
    // Initialize Ethereum connection
    const ethereumReady = await initializeEthereum();
    
    if (!ethereumReady) {
      logger.error('Failed to initialize Ethereum connection. Server will start but faucet functionality will be limited.');
    }
    
    app.listen(PORT, () => {
      logger.info(`Polymera EVM Faucet started on port ${PORT}`, {
        port: PORT,
        ethereumReady,
        faucetAddress: faucetAddress || 'unknown'
      });
      
      console.log(`🚰 Polymera EVM Faucet running on http://localhost:${PORT}`);
      console.log(`📊 Health check: http://localhost:${PORT}/health`);
      console.log(`💰 Faucet info: http://localhost:${PORT}/info`);
    });
  } catch (error) {
    logger.error('Failed to start server', { error: error.message });
    process.exit(1);
  }
}

// Graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  process.exit(0);
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  process.exit(0);
});

// Start the server
startServer();
