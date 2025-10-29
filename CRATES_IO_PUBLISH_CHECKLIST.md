# Crates.io 发布检查清单

## ✅ 发布前准备

### 1. Cargo.toml 配置
- [x] 包名: `dlsite-gamebox` (避免与原始 `dlsite` 冲突)
- [x] 版本: `0.2.0`
- [x] 描述: "High-performance DLsite client with caching, parallel parsing, and streaming support"
- [x] License: MIT
- [x] Repository: https://github.com/SuperToolman/dlsite-gamebox
- [x] Authors: SuperToolman
- [x] Keywords: dlsite, scraper, api, async, performance
- [x] Categories: api-bindings

### 2. 代码质量
- [x] 所有测试通过 (27/27 单元测试)
- [x] 编译无错误
- [x] 文档完整
- [x] 示例清晰

### 3. 文档
- [x] README.md 完整
- [x] CHANGELOG.md 完整
- [x] 代码注释充分
- [x] 示例代码可运行

### 4. 许可证
- [x] LICENSE 文件存在
- [x] Cargo.toml 中指定 MIT 许可证

### 5. 依赖
- [x] 所有依赖都是公开的
- [x] 没有本地路径依赖
- [x] 依赖版本合理

## 📋 发布步骤

### 步骤 1: 验证包
```bash
cargo publish --dry-run
```

### 步骤 2: 提交更改
```bash
git add Cargo.toml
git commit -m "chore: Update package name to dlsite-gamebox for crates.io"
git push origin master
```

### 步骤 3: 发布到 crates.io
```bash
cargo publish
```

### 步骤 4: 验证发布
访问: https://crates.io/crates/dlsite-gamebox

## 🔍 发布前验证

### 检查 Cargo.toml
```toml
[package]
name = "dlsite-gamebox"
version = "0.2.0"
edition = "2021"
description = "High-performance DLsite client with caching, parallel parsing, and streaming support"
license = "MIT"
repository = "https://github.com/SuperToolman/dlsite-gamebox"
authors = ["SuperToolman"]
keywords = ["dlsite", "scraper", "api", "async", "performance"]
categories = ["api-bindings"]
```

### 检查文件
- [x] Cargo.toml - 配置正确
- [x] Cargo.lock - 存在（可选）
- [x] src/lib.rs - 公开 API 正确
- [x] README.md - 包含使用示例
- [x] LICENSE - MIT 许可证

## 📊 包信息

| 项目 | 值 |
|------|-----|
| 包名 | dlsite-gamebox |
| 版本 | 0.2.0 |
| 许可证 | MIT |
| 仓库 | https://github.com/SuperToolman/dlsite-gamebox |
| 分类 | api-bindings |
| 关键词 | dlsite, scraper, api, async, performance |

## 🎯 发布后

### 1. 创建 GitHub Release
```bash
git tag -a v0.2.0-crates -m "Release v0.2.0 to crates.io"
git push origin v0.2.0-crates
```

### 2. 更新文档
- 在 README 中添加 crates.io 链接
- 在 GitHub 中创建 Release 说明

### 3. 宣传
- 在 Rust 社区分享
- 更新相关文档

## ⚠️ 注意事项

1. **包名冲突**: 已改为 `dlsite-gamebox` 以避免与原始 `dlsite` 冲突
2. **版本号**: 从 0.2.0 开始，表示这是一个优化版本
3. **向后兼容**: 所有 API 都是向后兼容的
4. **文档**: 确保所有公开 API 都有文档

## 🚀 快速发布命令

```bash
# 1. 验证包（推荐先运行）
cargo publish --dry-run

# 2. 发布到 crates.io
cargo publish

# 3. 验证发布成功
curl https://crates.io/api/v1/crates/dlsite-gamebox
```

## 📝 发布后的更新

发布后，如果需要更新：
1. 修改 Cargo.toml 中的版本号
2. 更新 CHANGELOG.md
3. 提交并推送到 GitHub
4. 运行 `cargo publish`

## ✨ 完成

准备好发布了！运行以下命令：

```bash
cargo publish
```

