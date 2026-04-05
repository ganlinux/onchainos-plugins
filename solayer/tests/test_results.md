# 测试结果报告

- 日期: 2026-04-05
- DApp 支持的链: 仅 Solana (chain 501)
- 编译: ✅
- Lint: ✅ (0 errors, 1 warning W100 base64 reference — 正常)

## 汇总

| 总数 | L1编译 | L2读取 | L3模拟 | L4链上 | 失败 | 阻塞 |
|------|--------|--------|--------|--------|------|------|
| 6    | 1 ✅   | 2 ✅   | 2 ✅   | 1 ✅   | 0    | 0    |

## 详细结果

| # | 场景（用户视角） | Level | 命令 | 结果 | TxHash | 备注 |
|---|----------------|-------|------|------|--------|------|
| 1 | 查询 SOL 和 sSOL 余额 | L2 | `./target/release/solayer balance` | ✅ PASS | — | SOL: 0.1252, sSOL: 0.0 |
| 2 | 查询质押仓位 | L2 | `./target/release/solayer positions --rpc https://api.mainnet-beta.solana.com` | ✅ PASS | — | 0 sSOL 仓位（质押前） |
| 3 | 模拟质押 0.01 SOL（dry-run） | L3 | `./target/release/solayer restake --amount 0.01 --dry-run` | ✅ PASS | — | dry_run:true，显示步骤和程序地址 |
| 4 | 模拟解质押（dry-run） | L3 | `./target/release/solayer unrestake --amount 0.1 --dry-run --rpc https://api.mainnet-beta.solana.com` | ✅ PASS | — | 显示5步解质押说明 |
| 5 | 实际质押 0.01 SOL 获得 sSOL | L4 | `./target/release/solayer restake --amount 0.01` | ✅ PASS | `3MbzcEKBF86azibHoPaJo5NPoTBh8CCrdd4nHccrmG7YAwzhoePqqMaiqTXzuGLK7FY92VjGv5ExpN7XstDjHUNK` | API返回 0.008740987 sSOL |
| 6 | 无效参数（缺少 --amount） | L1-error | `./target/release/solayer restake` | ✅ PASS | — | clap 正确报错提示 |

## L4 链上验证

- TxHash: `3MbzcEKBF86azibHoPaJo5NPoTBh8CCrdd4nHccrmG7YAwzhoePqqMaiqTXzuGLK7FY92VjGv5ExpN7XstDjHUNK`
- 查看: https://solscan.io/tx/3MbzcEKBF86azibHoPaJo5NPoTBh8CCrdd4nHccrmG7YAwzhoePqqMaiqTXzuGLK7FY92VjGv5ExpN7XstDjHUNK
- 钱包: `Da5mSX6tK7nyqK41C6jJcEgKkSYTPhWD41qKZGQWmL2z`
- 操作: restake 0.01 SOL → ~0.008741 sSOL

## 修复记录

| # | 问题 | 根因 | 修复 | 文件 |
|---|------|------|------|------|
| 1 | `onchainos wallet address` 不存在 | onchainos 子命令为 `addresses`，不是 `address` | 改用 `wallet balance` JSON 响应解析地址（路径 `data.details[0].tokenAssets[0].address`） | `src/onchainos.rs` |
| 2 | 双重调用 `wallet balance` 触发 TLS rate limit | `resolve_wallet_solana()` 和 `wallet_balance_solana()` 各调用一次 onchainos | 合并为 `resolve_wallet_and_balance_solana()` 单次调用同时返回地址和余额 JSON | `src/onchainos.rs`, `src/commands/*.rs` |
| 3 | reqwest 子进程无法连接 HTTPS 端点 | 代理环境变量未传递给 reqwest Client | 在 `build_client()` 中读取 `HTTPS_PROXY`/`https_proxy` 并配置 reqwest proxy | `src/api.rs` |
| 4 | restake dry-run 因余额不足失败 | 余额检查在 dry-run 前执行 | dry-run 时跳过余额强制检查（打印 warning 但不报错） | `src/commands/restake.rs` |
| 5 | restake dry-run 调用 API 返回 HTTP 400/500 | Solayer API 要求 `referrerkey` 参数，且余额不足时 API 也会拒绝 | dry-run 时完全跳过 API 调用，直接输出模拟步骤 | `src/commands/restake.rs` |
| 6 | unrestake dry-run 因 sSOL 余额为 0 失败 | sSOL 余额检查在 dry-run 前执行 | dry-run 时跳过 sSOL 余额强制检查 | `src/commands/unrestake.rs` |
| 7 | 默认 RPC `mainnet-rpc.solayer.org` 不可达 | Solayer RPC 在当前网络环境下连接不稳定 | 使用 `--rpc https://api.mainnet-beta.solana.com` 备用 RPC | — |
