# Solayer Plugin Test Cases

Generated: 2026-04-05
Plugin: solayer | Chain: Solana only (chain 501)

| # | 场景 | Level | 命令 | 预期结果 | Priority |
|---|------|-------|------|---------|----------|
| 1 | 用户查询 SOL 和 sSOL 余额 | L2 | `./target/release/solayer balance` | 输出 SOL 和 sSOL 数量 | P0 |
| 2 | 用户查询质押仓位 | L2 | `./target/release/solayer positions` | 输出 sSOL 余额和 SOL 价值 | P0 |
| 3 | 模拟质押 0.01 SOL（dry-run） | L3 | `./target/release/solayer restake --amount 0.01 --dry-run` | dry_run 标识，serialized_tx 非空 | P0 |
| 4 | 实际质押少量 SOL（0.01 SOL） | L4 | `./target/release/solayer restake --amount 0.01` | 返回 txHash | P0 |
| 5 | 查询解质押说明（dry-run） | L3 | `./target/release/solayer unrestake --amount 0.1 --dry-run` | 输出操作说明 | P1 |
| 6 | 无效参数测试（缺少 amount） | L1-error | `./target/release/solayer restake` | 输出 CLI 错误提示 | P1 |
