# QMX Backend Library

[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.2.0-blue.svg)](https://github.com/H-Chris233/qmx_backend_lib)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

一个用 Rust 编写的高性能学生管理和财务管理库，专为教育培训机构设计。提供完整的学生信息管理、现金流记录、分期付款处理、会员管理和统计分析功能。

## ✨ 特性

- 🎓 **学生管理** - 完整的学生档案管理，包含课程、成绩、会员信息
- 💰 **财务管理** - 现金流跟踪、收支记录、财务统计
- 📅 **分期付款** - 灵活的分期付款计划管理，支持多种付款频率
- 👥 **会员系统** - 会员期限管理、状态检查、到期提醒
- 📊 **统计分析** - 仪表板统计、收入分析、学员表现统计
- 💾 **数据持久化** - 原子性文件操作，数据安全可靠
- 🔒 **类型安全** - 完全利用 Rust 的类型系统，编译时错误检查
- ⚡ **高性能** - 零成本抽象，内存安全，无垃圾回收

## 🚀 快速开始

### 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
qmx_backend_lib = "1.2.0"
```

### 基本使用

```rust
use qmx_backend_lib::*;
use chrono::{Utc, Duration};

fn main() -> anyhow::Result<()> {
    // 初始化系统
    let mut db = init::init()?;

    // 创建学生
    let mut student = Student::new();
    student
        .set_name("张三".to_string())
        .set_age(18)
        .set_class(Class::TenTry)
        .set_subject(Subject::Shooting)
        .add_ring(9.5);

    // 设置会员期限
    student.set_membership_dates(
        Some(Utc::now()),
        Some(Utc::now() + Duration::days(365))
    );

    let student_id = student.uid();
    db.student.insert(student);

    // 添加收入记录
    let mut cash = Cash::new(Some(student_id));
    cash.set_cash(1000);
    cash.set_note(Some("学费收入".to_string()));
    db.cash.insert(cash);

    // 生成统计报告
    let stats = get_dashboard_stats(&db.student, &db.cash)?;
    println!("学生总数: {}", stats.total_students);
    println!("总收入: {}", stats.total_revenue);

    // 保存数据
    save::save(db)?;
    
    Ok(())
}
```

## 📚 核心概念

### 学生管理

```rust
// 创建学生
let mut student = Student::new();
student
    .set_name("李四".to_string())
    .set_age(20)
    .set_class(Class::Month)
    .set_subject(Subject::Archery)
    .add_ring(8.5)
    .add_ring(9.0);

// 会员管理
student.set_membership_start_date(Utc::now());
student.set_membership_end_date(Utc::now() + Duration::days(30));

if student.is_membership_active() {
    println!("会员有效，剩余 {:?} 天", student.membership_days_remaining());
}
```

### 财务管理

```rust
// 收入记录
let mut income = Cash::new(Some(student_id));
income.set_cash(1500);
income.set_note(Some("月卡费用".to_string()));

// 支出记录  
let mut expense = Cash::new(None);
expense.set_cash(-200);
expense.set_note(Some("设备维护".to_string()));

// 分期付款
let installment = Cash::new_installment(
    Some(student_id),
    6000,                           // 总金额
    6,                              // 6期
    PaymentFrequency::Monthly,      // 每月
    Utc::now() + Duration::days(30), // 首次到期时间
    1,                              // 当前期数
    None                            // 自动生成计划ID
);
```

### 分期付款管理

```rust
// 查询逾期分期
let overdue = db.cash.get_overdue_installments();
for cash in overdue {
    println!("逾期分期: 学生ID={:?}, 金额={}", cash.student_id, cash.cash);
}

// 生成下期分期
let plan_id = 1001;
let next_due = Utc::now() + Duration::days(30);
let next_uid = db.cash.generate_next_installment(plan_id, next_due)?;

// 取消分期计划
let cancelled = db.cash.cancel_installment_plan(plan_id);
println!("取消了 {} 个分期记录", cancelled);
```

## 🏗️ 架构设计

### v1.2.0 新特性

- **通用Database<T> Trait**: 统一的数据库操作接口，消除重复代码
- **统一错误处理**: 全面使用 `anyhow::Result<T>` 提供丰富的错误上下文
- **优化API设计**: 更符合Rust最佳实践的接口设计
- **会员管理系统**: 完整的会员期限管理和状态检查
- **向后兼容性**: 现有代码无需修改即可升级

### 模块结构

```
qmx_backend_lib/
├── src/
│   ├── lib.rs          # 库入口点
│   ├── common.rs       # 通用数据库trait (新增)
│   ├── student.rs      # 学生和会员管理
│   ├── cash.rs         # 现金流和分期付款
│   ├── database.rs     # 数据库容器
│   ├── stats.rs        # 统计分析
│   ├── init.rs         # 系统初始化
│   └── save.rs         # 数据持久化
├── tests/              # 综合测试套件 (116个测试)
└── data/               # 数据存储目录 (运行时创建)
    ├── student_database.json
    ├── cash_database.json
    ├── uid_counter
    └── cash_uid_counter
```

### 数据流

```
[初始化] → [加载数据] → [业务操作] → [保存数据]
    ↓           ↓           ↓           ↓
  init()   Database   CRUD操作   save()
               ↓           ↓           ↓
         StudentDB   CashDB     JSON文件
```

## 📖 详细文档

- **[API 完整文档](API.md)** - 所有API的详细说明和示例
- **[开发者文档](CODEBUDDY.md)** - 开发环境配置和项目结构
- **[测试说明](#testing)** - 如何运行和编写测试

## 🧪 测试

本项目包含116个综合测试，覆盖所有核心功能：

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test student_tests
cargo test cash_tests
cargo test stats_tests

# 运行测试并显示输出
cargo test -- --nocapture
```

测试覆盖：
- ✅ 学生CRUD操作和会员管理
- ✅ 现金流记录和分期付款
- ✅ 数据持久化和错误处理
- ✅ 统计分析和边界情况
- ✅ 并发安全和数据一致性

## 🔧 开发

### 环境要求

- Rust 1.87+
- Cargo

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行示例
cargo run --example basic_usage
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open
```

## 📊 性能特点

- **内存效率**: 使用 `BTreeMap` 提供有序存储和高效查询
- **原子操作**: UID计数器使用原子操作确保线程安全
- **零拷贝**: API设计避免不必要的数据拷贝
- **批量操作**: 支持批量插入、更新、删除操作
- **延迟加载**: 按需加载数据，减少内存占用

## 🔒 安全性

- **类型安全**: 利用Rust类型系统防止运行时错误
- **内存安全**: 无空指针、无缓冲区溢出、无内存泄漏
- **数据完整性**: 原子文件操作确保数据一致性
- **输入验证**: 严格的数据验证防止无效输入
- **错误处理**: 全面的错误处理，无panic风险

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 贡献指南

- 遵循现有的代码风格
- 添加测试覆盖新功能
- 更新相关文档
- 确保所有测试通过

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🔗 相关链接

- [GitHub Repository](https://github.com/H-Chris233/qmx_backend_lib)
- [API Documentation](API.md)
- [Issue Tracker](https://github.com/H-Chris233/qmx_backend_lib/issues)

## 📞 支持

如果你遇到问题或有建议，请：

1. 查看 [API文档](API.md)
2. 搜索 [已知问题](https://github.com/H-Chris233/qmx_backend_lib/issues)
3. 创建新的 [Issue](https://github.com/H-Chris233/qmx_backend_lib/issues/new)

---

**QMX Backend Library** - 让教育培训机构管理更简单、更高效！ 🎓✨