// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Pool {
    address public owner; // 资金池所有者

    event Withdraw(address indexed to, uint256 amount);

    constructor(address _owner) {
        owner = _owner; // 设置资金池所有者为主合约地址
    }

    // 接收存款
    receive() external payable {}

    // 仅允许主合约提取资金
    function withdraw(address to, uint256 amount) external {
        require(msg.sender == owner, "Only owner can withdraw");
        require(amount <= address(this).balance, "Insufficient balance in pool");
        payable(to).transfer(amount);
        emit Withdraw(to, amount);
    }
}
