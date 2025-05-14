// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IWormholeRelayer {
    function receiveVAA(bytes calldata vaa) external;
}

contract Vault {
    address public owner;

    mapping(address => uint256) public ethCollateral;
    mapping(address => bool) public hasActiveLoan;

    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event VAAReceived(bytes vaa);

    constructor() {
        owner = msg.sender;
    }

    // Allow user to deposit ETH as collateral
    function deposit() external payable {
        require(msg.value > 0, "No ETH sent");
        ethCollateral[msg.sender] += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    // Allow user to withdraw ETH if no active loan
    function withdraw(uint256 amount) external {
        require(!hasActiveLoan[msg.sender], "Active loan exists");
        require(ethCollateral[msg.sender] >= amount, "Insufficient collateral");

        ethCollateral[msg.sender] -= amount;
        payable(msg.sender).transfer(amount);
        emit Withdrawn(msg.sender, amount);
    }

    // Called by relayer to post cross-chain message from Solana
    function receiveVAA(bytes calldata vaa) external {
        // NOTE: In real usage, parse and verify VAA with Wormhole
        emit VAAReceived(vaa);

        // For MVP: simulate parsing
        (address user, string memory action) = parseVAA(vaa);

        if (keccak256(bytes(action)) == keccak256("repay")) {
            hasActiveLoan[user] = false;
        } else if (keccak256(bytes(action)) == keccak256("liquidate")) {
            // Liquidator takes ETH
            ethCollateral[user] = 0;
        }
    }

    // Stub: simulate ETH to USDC swap
    function swapETHtoUSDC(uint256 ethAmount) internal pure returns (uint256) {
        // Simulate 1 ETH = 2000 USDC
        return ethAmount * 2000;
    }

    // Stub: simulate bridging USDC via Wormhole
    function bridgeUSDCToSolana(address user, uint256 usdcAmount) internal {
        // Simulated bridge logic
        // Real impl would use Wormhole Token Bridge
    }

    // Simulate parsing VAA (very basic!)
    function parseVAA(bytes calldata vaa) internal pure returns (address, string memory) {
        // In MVP, fake it: assume VAA is [20 bytes addr | rest string "repay"/"liquidate"]
        address user = address(bytes20(vaa[0:20]));
        string memory action = string(vaa[20:]);
        return (user, action);
    }
}
