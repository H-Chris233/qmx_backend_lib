# 变更日志

所有重要的项目变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [2.2.0] - 2025-09-15

### 🚨 破坏性变更

#### CashBuilder API 变更
- **BREAKING**: `CashBuilder::build()` 现在返回 `Result<Cash>` 而不是 `Cash`
- **BREAKING**: 金额为 0 时会返回错误: `"amount cannot be zero"`
- **BREAKING**: `CashUpdater::amount()` 现在会在运行时校验金额不能为0

```rust
// 迁移示例
// 旧代码 (v2.1.x)
let cash = CashBuilder::new(amount).build();

// 新代码 (v2.2.0+)
let cash = CashBuilder::new(amount).build()?;  // 需要错误处理
```

#### TimePeriod 行为变更
- **BREAKING**: `FinancialStats::calculate()` 现在真正按 `TimePeriod` 过滤数据
- 之前 `TimePeriod` 参数被忽略，总是返回全部时间的统计
- 现在各时间周期会实际过滤对应时间范围的数据

```rust
// 如果需要全部时间的统计，必须显式指定
let all_stats = manager.get_financial_stats(TimePeriod::AllTime)?;
```

### ✨ 新增功能

#### 字段清空支持
- `StudentUpdater` 现在支持用 `None` 显式清空 Optional 字段
- 支持清空 `lesson_left` 和 `membership` 字段

```rust
// 清空课时和会员信息
manager.update_student(uid, 
    StudentUpdater::new()
        .lesson_left(None)        // 清空课时字段
        .membership(None, None)   // 清空会员信息
)?;
```

#### TimePeriod 增强
- 新增 `TimePeriod::Today` - 今日统计
- 新增 `TimePeriod::Custom { start, end }` - 自定义时间范围统计

```rust
let today_stats = manager.get_financial_stats(TimePeriod::Today)?;
let custom_stats = manager.get_financial_stats(TimePeriod::Custom {
    start: start_date,
    end: end_date,
})?;
```

#### 环境变量配置
- 新增 `QMX_DATA_DIR` 环境变量支持
- 可通过环境变量配置数据目录路径

```rust
std::env::set_var("QMX_DATA_DIR", "/custom/data/path");
let manager = QmxManager::new(true)?;  // 使用自定义路径
```

#### Student API 增强
- 新增 `Student::set_class_with_lesson_init()` 方法
- 分离了 `set_class()` 的副作用，提供更清晰的API设计
- 新增 `StudentUpdater::clear_lesson_left()` 便捷方法

### 🔧 改进

#### 错误处理优化
- 移除了 JSON 格式的错误嵌入，改用标准日志记录
- `Database::save_to_string()` 失败时返回空字符串而不是JSON错误
- 增强了错误上下文信息

#### 性能优化
- **延迟克隆**: 查询接口延迟克隆操作到API边界
- 减少了不必要的数据复制，提升查询性能
- 优化了 `StudentQuery` 和 `CashQuery` 的内存使用

#### 数据一致性增强
- 使用 `tempfile` crate 替换手写的原子写入操作
- 增加 `fsync` 和目录同步支持，提升崩溃一致性
- UID 计数器写入现在包含完整的持久化保证

#### 代码质量
- 统一了错误处理模式，使用 `anyhow::Context`
- 改进了模块间的职责分离
- 增强了输入验证和类型安全

### 🐛 修复

#### 功能修复
- 修复了 `TimePeriod` 在财务统计中被忽略的问题
- 修复了 `StudentUpdater` 无法清空 Optional 字段的问题
- 修复了原子写入在某些平台上的兼容性问题

#### 测试修复
- 更新了所有受API变更影响的测试用例
- 修复了 `test_student_set_class_with_lesson_init` 测试
- 确保测试覆盖新的错误处理路径

### 📚 文档更新

- 更新了 `API_v2.md` 以反映所有API变更
- 创建了 `API_CHANGES_v2.2.0.md` 详细迁移指南
- 增加了破坏性变更的详细说明和迁移示例
- 更新了所有代码示例以使用新的API

### 🔒 安全性

- 增强了文件写入的原子性，防止数据损坏
- 改进了错误信息的处理，避免敏感信息泄露
- 加强了输入验证，防止无效数据写入

---

## [2.1.0] - 2025-09-14

### ✨ 新增功能
- 完整的 v2 API 实现
- 统一的 `QmxManager` 入口
- Builder、Updater、Query 模式支持
- 线程安全的设计架构

### 🔧 改进
- 向后兼容 v1 API
- 增强的统计分析功能
- 改进的错误处理

---

## [2.0.0] - 2025-09-13

### 🚨 破坏性变更
- 引入全新的 v2 API 架构
- 重构了核心数据结构

### ✨ 新增功能
- 现代化的 API 设计
- 改进的类型安全
- 增强的查询能力

---

## [1.0.0] - 2025-09-12

### ✨ 初始发布
- 基础的学生管理功能
- 现金记录系统
- 简单的统计分析
- JSON 数据持久化

---

## 迁移指南

### 从 v2.1.x 升级到 v2.2.0

1. **更新 CashBuilder 调用**:
   ```rust
   // 旧代码
   let cash = CashBuilder::new(amount).build();
   
   // 新代码
   let cash = CashBuilder::new(amount).build()?;
   ```

2. **检查 TimePeriod 使用**:
   ```rust
   // 如果需要全部数据，显式指定
   let stats = manager.get_financial_stats(TimePeriod::AllTime)?;
   ```

3. **利用新的字段清空功能**:
   ```rust
   // 现在可以清空 Optional 字段
   manager.update_student(uid, 
       StudentUpdater::new().lesson_left(None)
   )?;
   ```

4. **配置数据目录**:
   ```rust
   // 可选：使用环境变量配置
   std::env::set_var("QMX_DATA_DIR", "/your/data/path");
   ```

### 从 v1.x 升级到 v2.x

请参考 `API_v2.md` 中的详细迁移指南。v2 API 完全向后兼容，可以渐进式迁移。

---

*更多详细信息请参考对应版本的 API 文档。*