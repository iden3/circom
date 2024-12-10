// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./pool.sol";

contract MultiPoolMixer {
    Pool[] public pools; // 动态生成的资金池列表
    address public admin; // 合约管理员

    event PoolCreated(address indexed poolAddress);
    event Deposit(address indexed from, uint256 amount, address pool);
    event Withdrawal(address indexed to, uint256 amount, address pool);
    event WithdrawalAttempt(address indexed poolAddress, uint256 poolBalance, address indexed recipient); // 新增事件

    constructor() {
        admin = msg.sender; // 设置管理员为合约部署者
    }

    // 一次性创建多个资金池
    function createPools(uint256 numberOfPools) external onlyAdmin {
        require(numberOfPools > 0, "Number of pools must be greater than zero");
        for (uint256 i = 0; i < numberOfPools; i++) {
            Pool newPool = new Pool(address(this));
            pools.push(newPool);
            emit PoolCreated(address(newPool));
        }
    }

    // 用户存款
    function deposit() external payable {
        uint256 requiredDeposit = 10 * 10**18; // 固定存款金额为 10 XRP，假设 1 XRP = 10^18 wei
        require(msg.value == requiredDeposit, "Deposit amount must be exactly 10 XRP");
        require(pools.length > 0, "No pools available");

        // 随机选择一个资金池
        uint256 randomIndex = uint256(keccak256(abi.encodePacked(block.timestamp, block.prevrandao, msg.sender))) % pools.length;
        address selectedPool = address(pools[randomIndex]);

        // 转移资金到随机池
        (bool success, ) = selectedPool.call{value: msg.value}("");
        require(success, "Transfer to pool failed");

        emit Deposit(msg.sender, msg.value, selectedPool);
    }

    function withdraw(address payable to) external {
        require(pools.length > 0, "No pools available");
        require(to != address(0), "Invalid recipient address");

        uint256 fixedWithdrawAmount = 10 * 10**18; // 固定提取金额为 10 XRP
        uint256 poolCount = pools.length;

        for (uint256 i = 0; i < poolCount; i++) {
            uint256 randomIndex = uint256(keccak256(abi.encodePacked(block.timestamp, block.prevrandao, msg.sender, i))) % poolCount;

            // 将 selectedPool 定义为 address payable
            address payable selectedPool = payable(address(pools[randomIndex]));

            uint256 poolBalance = selectedPool.balance;

            emit WithdrawalAttempt(selectedPool, poolBalance, to); // 调试信息

            if (poolBalance >= fixedWithdrawAmount) {
                // 调用资金池的 withdraw 函数
                Pool(selectedPool).withdraw(to, fixedWithdrawAmount);
                emit Withdrawal(to, fixedWithdrawAmount, selectedPool);
                return;
            }
        }

        revert("No pool with sufficient balance available");
    }

    // 获取十个资金池内的总资金
    function getTotalFundsInFirstTenPools() external view returns (uint256) {
        uint256 totalFunds = 0;
        uint256 limit = pools.length < 10 ? pools.length : 10; // 确保最多只检查十个池
        for (uint256 i = 0; i < limit; i++) {
            totalFunds += address(pools[i]).balance; // 获取资金池的余额
        }
        return totalFunds;
    }

    // 获取当前所有资金池地址
    function getPools() external view returns (address[] memory) {
        address[] memory poolAddresses = new address[](pools.length);
        for (uint256 i = 0; i < pools.length; i++) {
            poolAddresses[i] = address(pools[i]);
        }
        return poolAddresses;
    }

    // 限制管理员权限
    modifier onlyAdmin() {
        require(msg.sender == admin, "Only admin can call this function");
        _;
    }
}
