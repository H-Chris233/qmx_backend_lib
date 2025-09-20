# QMX Backend Library v2.5.0 变更说明

## 概述

v2.5.0 是一个重要的更新版本，主要针对年龄字段进行了重构，使其支持可空值。此变更提高了数据模型的准确性并更好地反映了业务需求。

---

## 🔧 破坏性变更

### Student.age 字段类型变更

**变更内容：** `Student.age` 字段从 `u8` 类型更改为 `Option<u8>` 类型

**变更原因：** 
- 更好地表示"未知年龄"或"未提供年龄"的状态
- 避免使用特殊值（如0）来表示无意义的业务状态
- 提高数据模型的准确性和代码的可读性

**迁移影响：**
所有直接使用 `Student.age()` 方法的地方都需要更新，因为返回类型从 `u8` 变更为 `Option<u8>`。

**迁移示例：**

```rust
// 旧代码 (v2.4.1 及之前)
let student = Student::new();
student.set_age(18);
let age: u8 = student.age();  // 直接返回 u8 值

// 新代码 (v2.5.0+)
let student = Student::new();
student.set_age(Some(18));    // 设置年龄时需要包装在 Some 中
let age: Option<u8> = student.age();  // 返回 Option<u8>

// 如果需要获取具体的年龄值，需要处理 Option
if let Some(age_value) = student.age() {
    println!("学生年龄: {}", age_value);
} else {
    println!("学生年龄未知");
}
```

### StudentBuilder 和 StudentUpdater API 变更

**变更内容：** 
- `StudentBuilder::new(name, age)` 的签名从 `new(name: impl Into<String>, age: Option<u8>)` 更改为 `new(name: impl Into<String>)`
- 新增 `StudentBuilder::age(age: u8)` 方法用于链式设置年龄
- `StudentUpdater::age(age)` 的参数从 `u8` 更改为 `Option<u8>`

**变更原因：**
- 统一API设计模式，使所有字段都通过链式调用方式设置
- 提高API的一致性和易用性
- 避免构造函数参数列表过长

**迁移示例：**

```rust
// 旧代码 (v2.4.1 及之前)
let builder = StudentBuilder::new("张三", Some(18));
let updater = StudentUpdater::new().age(19);

// 新代码 (v2.5.0+)
let builder = StudentBuilder::new("张三").age(18);  // 设置具体年龄
let builder = StudentBuilder::new("李四");          // 不设置年龄（保持None）
let updater = StudentUpdater::new().age(Some(19));  // 设置具体年龄
let updater = StudentUpdater::new().age(None);      // 清除年龄信息
```

### Student::set_age 方法变更

**变更内容：** `Student::set_age(age)` 的参数从 `u8` 更改为 `Option<u8>`

**迁移示例：**

```rust
// 旧代码 (v2.4.1 及之前)
student.set_age(20);

// 新代码 (v2.5.0+)
student.set_age(Some(20));  // 设置具体年龄
student.set_age(None);      // 清除年龄信息
```

---

## 📚 文档更新

所有相关的API文档都已更新以反映这些变更：
- `API_v1.md` - 更新了Student结构体定义和方法签名
- `API_v2.md` - 更新了StudentBuilder和StudentUpdater的API说明及示例

---

## 🧪 测试更新

所有相关的测试用例都已更新以适配新的API：
- `student_tests.rs` - 更新了所有年龄相关测试
- `v2_api_tests.rs` - 更新了V2 API测试用例
- `v1_api_tests.rs` - 更新了V1 API测试用例
- `enhanced_tests.rs` - 更新了增强测试用例
- `atomic_write_fix_tests.rs` - 更新了原子写入测试用例

---

## 🎯 使用建议

1. **迁移策略：** 建议在代码中逐步替换所有对年龄字段的使用，确保正确处理 `Option<u8>` 类型
2. **空值处理：** 在业务逻辑中明确处理年龄未知的情况
3. **向后兼容：** 此变更为破坏性变更，需要相应地更新所有依赖此库的代码
4. **API一致性：** 建议使用新的链式调用模式设置所有学生属性，以保持代码风格的一致性
5. **可读性提升：** 新的API模式使代码更具可读性，每个属性的设置都清晰明确

---

*文档版本：2.5.0*  
*最后更新：2025-09-20*