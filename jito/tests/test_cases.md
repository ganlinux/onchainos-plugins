# Jito Plugin — 测试用例

- 日期: 2026-04-05
- DApp 支持的链: Solana only (chain 501)
- 插件版本: 0.1.0

---

## L2 — 读取测试（无需钱包签名）

| # | 命令 | 参数 | 预期结果 |
|---|------|------|---------|
| L2-1 | `jito info` | — | 输出包含 APY、SOL Price、JitoSOL Supply、Stake Pool 地址、JitoSOL Mint 地址 |
| L2-2 | `jito positions` | — | 输出包含钱包地址、JitoSOL Balance、APY（可为 0 余额） |
| L2-3 | `jito restake-vaults` | — | 输出包含 Vault 地址列表或 fallback 链接 jito.network |

---

## L3 — Dry-Run 模拟测试（不上链）

| # | 命令 | 参数 | 预期结果 |
|---|------|------|---------|
| L3-1 | `jito --dry-run stake` | `--amount 0.01` | 输出 `[dry-run]` 提示、serializedData 长度、合约调用预览 |
| L3-2 | `jito --dry-run unstake` | `--amount 0.001` | 输出 `[dry-run]` 提示、swap 命令预览（JitoSOL → SOL） |
| L3-3 | `jito --dry-run restake-deposit` | `--vault Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb --amount 0.01` | 输出 `[dry-run]` 提示、MintTo 指令预览 |
| L3-4 | `jito --dry-run restake-withdraw` | `--vault Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb --amount 0.01` | 输出 `[dry-run]` 提示、EnqueueWithdrawal 指令预览 |

---

## L4 — 链上写操作（Solana mainnet）

| # | 命令 | 参数 | 预期结果 | 最小金额 |
|---|------|------|---------|---------|
| L4-1 | `jito stake` | `--amount 0.01` | 输出 `txHash`（非空、非 pending）；JitoSOL 出现在 positions | 0.01 SOL（代码强制） |
| L4-2 | `jito unstake` | `--amount 0.001` | 输出 `txHash`；SOL 余额增加（L4-1 完成后执行） | 0.001 JitoSOL |

### L4 前置条件

- 钱包地址: `Da5mSX6tK7nyqK41C6jJcEgKkSYTPhWD41qKZGQWmL2z`
- 链: Solana mainnet (chain 501)
- 所需余额: >= 0.015 SOL（0.01 stake + 0.005 gas）
- 锁: `phase3/jito`（acquire-lock.sh / release-lock.sh）

---

## 关键地址

| 名称 | 地址 |
|------|------|
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |
| SPL Stake Pool Program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` |
| Jito Stake Pool Account | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` |
| Jito Vault Program | `Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8` |
| onchainos investmentId | `22414` |
