# Lotus-Sign

Filecoin 本地签名工具 - Rust 实现

## 概述

`lotus-sign` 是一款安全的 Filecoin 钱包工具，所有签名操作均在本地完成，私钥永不上传。

**核心功能：**
- 钱包创建、导入、导出
- FIL 转账签名
- 矿工 Owner/Worker 管理
- 矿工余额提现

**安全特性：**
- 私钥 AES-256-GCM 加密存储
- 支持离线签名
- SQLite 本地数据库

## 快速开始

### 编译安装

```bash
cd lotus-sign
cargo build --release
```

### 配置文件

创建 `~/lotus-sign/config.toml`：

```toml
[api]
url = "https://api.node.glif.io/rpc/v1"

[wallet]
password = "your-password"
```

## 命令参考

### 钱包操作

```bash
# 创建钱包
lotus-sign wallet new secp256k1    # f1 地址
lotus-sign wallet new bls          # f3 地址

# 导入私钥
lotus-sign wallet import <私钥hex>

# 导出私钥
lotus-sign wallet export <地址>

# 查看列表
lotus-sign wallet list

# 查询余额
lotus-sign wallet balance <地址>
```

### 转账

```bash
lotus-sign send <目标地址> <金额> --from <发送地址>

# 示例
lotus-sign send f1xxx 0.1 --from f1yyy
```

### 矿工管理

```bash
# 查看矿工信息
lotus-sign actor info <矿工地址>

# 提现
lotus-sign withdraw --miner <矿工> --amount <金额> --from <owner>
```

## 技术规格

| 项目 | 说明 |
|-----|------|
| 签名算法 | secp256k1 / BLS12-381 |
| 加密方式 | AES-256-GCM |
| 哈希算法 | Blake2b |
| 序列化 | CBOR |
| 存储 | SQLite |

## 项目结构

```
src/
├── cli/          # 命令行接口
├── chain/        # 链数据结构
├── wallet/       # 钱包与签名
├── crypto/       # 加密模块
├── db/           # 数据库
├── config/       # 配置管理
└── service/      # RPC 服务
```

## 许可证

MIT
