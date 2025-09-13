use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::common::{Database, HasUid};

pub static CASH_UID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 独立的 Cash 结构体，包含自己的 UID 和关联的学生 ID
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cash {
    /// Cash 自己的唯一标识符
    pub uid: u64,
    /// 关联的学生 UID
    pub student_id: Option<u64>,
    /// 金额
    pub cash: i64,
    /// 备注信息
    pub note: Option<String>,
    /// 分期付款信息
    pub installment: Option<Installment>,
    /// 创建时间戳
    pub created_at: DateTime<Utc>,
}

/// 分期付款计划（新增）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installment {
    /// 分期计划ID（同一计划的各期共享相同ID）
    pub plan_id: u64,
    /// 总金额
    pub total_amount: i64,
    /// 总期数
    pub total_installments: u32,
    /// 当前期数
    pub current_installment: u32,
    /// 付款频率
    pub frequency: PaymentFrequency,
    /// 到期日期
    pub due_date: DateTime<Utc>,
    /// 付款状态
    pub status: InstallmentStatus,
}

/// 付款频率枚举（新增）
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentFrequency {
    Weekly,
    Monthly,
    Quarterly,
    Custom(u32), // 自定义天数
}

/// 分期付款状态枚举（新增）
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallmentStatus {
    Pending,
    Paid,
    Overdue,
    Cancelled,
}

impl Default for InstallmentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl Cash {
    pub fn new(student_id: Option<u64>) -> Self {
        let uid = CASH_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let new_cash = Self {
            uid,
            student_id,
            cash: 0,
            note: None,
            installment: None, // 默认没有分期
            created_at: Utc::now(),
        };
        info!("创建新的Cash记录，UID为: {}", new_cash.uid);
        new_cash
    }

    /// 创建新的分期付款记录
    pub fn new_installment(
        student_id: Option<u64>,
        total_amount: i64,
        total_installments: u32,
        frequency: PaymentFrequency,
        due_date: DateTime<Utc>,
        current_installment: u32,
        plan_id: Option<u64>,
    ) -> Self {
        let uid = CASH_UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        // 分期金额计算：每期基础金额 = 总金额 / 总期数
        // 最后一期加上余数，确保总金额正确
        let base_amount = total_amount / total_installments as i64;
        let remainder = total_amount % total_installments as i64;
        let cash = base_amount
            + if current_installment == total_installments {
                remainder
            } else {
                0
            };

        let plan_id = plan_id.unwrap_or_else(|| CASH_UID_COUNTER.fetch_add(1, Ordering::SeqCst));

        let cash_record = Cash {
            uid,
            student_id,
            cash,
            note: None,
            installment: Some(Installment {
                plan_id,
                total_amount,
                total_installments,
                current_installment,
                frequency,
                due_date,
                status: InstallmentStatus::Pending,
            }),
            created_at: Utc::now(),
        };

        // 添加分期创建日志
        info!(
            "创建分期付款记录: UID={}, 计划ID={}, 期数={}/{}, 金额={}, 到期时间={}",
            uid, plan_id, current_installment, total_installments, cash_record.cash, due_date
        );

        cash_record
    }

    pub fn add(&mut self, num: i64) {
        self.cash += num;
    }

    pub fn set_cash(&mut self, num: i64) {
        self.cash = num;
    }

    pub fn set_id(&mut self, id: u64) {
        self.student_id = if id == 0 { None } else { Some(id) };
    }

    /// 设置备注信息
    pub fn set_note(&mut self, note: Option<String>) {
        self.note = note;
    }

    /// 获取备注信息
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// 检查是否是分期付款（新增）
    pub fn is_installment(&self) -> bool {
        self.installment.is_some()
    }

    /// 获取分期计划ID（新增）
    pub fn installment_plan_id(&self) -> Option<u64> {
        self.installment.as_ref().map(|i| i.plan_id)
    }

    /// 更新分期付款状态
    pub fn set_installment_status(&mut self, status: InstallmentStatus) {
        if let Some(installment) = &mut self.installment {
            let old_status = installment.status;
            installment.status = status;
            info!(
                "更新分期付款状态: UID={}, 计划ID={}, 期数={}, 状态: {:?} -> {:?}",
                self.uid, installment.plan_id, installment.current_installment, old_status, status
            );
        }
    }
}

impl HasUid for Cash {
    fn uid(&self) -> u64 {
        self.uid
    }
}

/// Cash 数据库结构，支持持久化存储
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashDatabase {
    pub cash_data: BTreeMap<u64, Cash>,
}

impl Default for CashDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Database<Cash> for CashDatabase {
    fn data(&self) -> &BTreeMap<u64, Cash> {
        &self.cash_data
    }

    fn data_mut(&mut self) -> &mut BTreeMap<u64, Cash> {
        &mut self.cash_data
    }

    fn default_path(&self) -> &'static str {
        "./data/cash_database.json"
    }

    fn type_name(&self) -> &'static str {
        "现金"
    }

    fn static_type_name() -> &'static str {
        "现金"
    }

    fn new() -> Self {
        Self {
            cash_data: BTreeMap::new(),
        }
    }
}

impl CashDatabase {
    /// 从JSON字符串反序列化数据库
    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str).with_context(|| "从JSON反序列化现金数据库失败")
    }

    // 向后兼容性方法 - 直接委托给trait实现
    pub fn new() -> Self {
        <Self as Database<Cash>>::new()
    }

    pub fn insert(&mut self, cash: Cash) {
        <Self as Database<Cash>>::insert(self, cash)
    }

    pub fn insert_batch(&mut self, cash_records: Vec<Cash>) -> usize {
        <Self as Database<Cash>>::insert_batch(self, cash_records)
    }

    pub fn update_batch<F>(&mut self, uids: &[u64], update_fn: F) -> usize
    where
        F: FnMut(&mut Cash) -> bool,
    {
        <Self as Database<Cash>>::update_batch(self, uids, update_fn)
    }

    pub fn json(&self) -> String {
        <Self as Database<Cash>>::json(self)
    }

    pub fn get(&self, index: &u64) -> Option<&Cash> {
        <Self as Database<Cash>>::get(self, index)
    }

    pub fn save(&self) -> Result<()> {
        <Self as Database<Cash>>::save(self)
    }

    pub fn save_to(&self, path: &str) -> Result<()> {
        <Self as Database<Cash>>::save_to(self, path)
    }

    pub fn read_from(path: &str) -> Result<Self> {
        <Self as Database<Cash>>::read_from(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Cash)> + '_ {
        <Self as Database<Cash>>::iter(self)
    }

    pub fn len(&self) -> usize {
        <Self as Database<Cash>>::len(self)
    }

    pub fn is_empty(&self) -> bool {
        <Self as Database<Cash>>::is_empty(self)
    }

    pub fn remove(&mut self, uid: &u64) -> Option<Cash> {
        <Self as Database<Cash>>::remove(self, uid)
    }

    pub fn remove_batch(&mut self, uids: &[u64]) -> usize {
        <Self as Database<Cash>>::remove_batch(self, uids)
    }

    /// 获取所有分期付款记录（新增）
    pub fn get_installments(&self) -> Vec<&Cash> {
        self.cash_data
            .values()
            .filter(|c| c.installment.is_some())
            .collect()
    }

    /// 获取指定分期计划的所有记录（新增）
    pub fn get_installments_by_plan(&self, plan_id: u64) -> Vec<&Cash> {
        self.cash_data
            .values()
            .filter(|c| matches!(c.installment_plan_id(), Some(id) if id == plan_id))
            .collect()
    }

    /// 获取逾期分期付款（新增）
    pub fn get_overdue_installments(&self) -> Vec<&Cash> {
        let now = Utc::now();
        self.cash_data
            .values()
            .filter(|c| {
                if let Some(installment) = &c.installment {
                    installment.status == InstallmentStatus::Pending && installment.due_date < now
                } else {
                    false
                }
            })
            .collect()
    }

    /// 获取学生的分期付款记录（新增）
    pub fn get_student_installments(&self, student_id: u64) -> Vec<&Cash> {
        self.cash_data
            .values()
            .filter(|c| c.student_id == Some(student_id) && c.installment.is_some())
            .collect()
    }

    /// 生成下一期分期付款
    pub fn generate_next_installment(
        &mut self,
        plan_id: u64,
        due_date: DateTime<Utc>,
    ) -> Result<u64> {
        let installments = self.get_installments_by_plan(plan_id);
        if installments.is_empty() {
            error!("尝试生成下一期分期付款失败: 找不到计划ID {}", plan_id);
            return Err(anyhow::anyhow!("找不到分期计划 {}", plan_id));
        }

        let first = installments.first().expect("已检查 installments 非空");
        let installment_info = first
            .installment
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("计划ID {} 对应的记录不是分期付款记录", plan_id))?;

        // 计算当前最大期数
        // 安全：get_installments_by_plan 返回的记录都有 installment 信息
        let max_installment = installments
            .iter()
            .filter_map(|c| c.installment.as_ref().map(|i| i.current_installment))
            .max()
            .unwrap(); // 因为已经过滤，安全 unwrap

        // 检查是否已完成所有分期
        if max_installment >= installment_info.total_installments {
            warn!(
                "尝试生成下一期分期付款失败: 计划 {} 已完成 (当前期数 {}，总期数 {})",
                plan_id, max_installment, installment_info.total_installments
            );
            return Err(anyhow::anyhow!("分期计划 {} 已完成", plan_id));
        }

        let next_installment = max_installment + 1;

        // 创建新分期记录
        let new_cash = Cash::new_installment(
            first.student_id,
            installment_info.total_amount,
            installment_info.total_installments,
            installment_info.frequency,
            due_date,
            next_installment,
            Some(plan_id),
        );

        let uid = new_cash.uid;
        let cash = new_cash.cash;
        self.insert(new_cash);

        info!(
            "为计划 {} 生成第 {} 期分期付款: UID={}, 金额={}, 到期时间={}",
            plan_id, next_installment, uid, cash, due_date
        );

        Ok(uid)
    }

    /// 取消指定分期计划的所有未完成付款
    ///
    /// # 参数
    /// * `plan_id` - 要取消的分期计划ID
    ///
    /// # 返回值
    /// 返回被取消的付款记录数量
    pub fn cancel_installment_plan(&mut self, plan_id: u64) -> usize {
        let mut cancelled_count = 0;

        // 查找指定计划的所有分期记录
        for cash in self.cash_data.values_mut() {
            if let Some(installment) = &mut cash.installment {
                if installment.plan_id == plan_id {
                    // 只取消未完成的付款（Pending 或 Overdue 状态）
                    if installment.status == InstallmentStatus::Pending
                        || installment.status == InstallmentStatus::Overdue
                    {
                        let old_status = installment.status;
                        installment.status = InstallmentStatus::Cancelled;
                        cancelled_count += 1;

                        info!(
                            "取消分期付款: UID={}, 计划ID={}, 期数={}, 状态: {:?} -> Cancelled",
                            cash.uid, plan_id, installment.current_installment, old_status
                        );
                    }
                }
            }
        }

        if cancelled_count > 0 {
            info!(
                "成功取消分期计划 {} 中的 {} 个未完成付款",
                plan_id, cancelled_count
            );
        } else {
            warn!(
                "尝试取消分期计划 {}，但未找到任何可取消的未完成付款",
                plan_id
            );
        }

        cancelled_count
    }
}

/// 加载已保存的 Cash UID 计数器
pub fn load_saved_cash_uid() -> Result<u64> {
    let path = "./data/cash_uid_counter";
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .trim()
            .parse::<u64>()
            .inspect(|&uid| {
                info!("成功加载CASH UID: {}", uid);
            })
            .with_context(|| format!("解析路径为 '{}' 的CASH UID失败", path)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("未找到现有的CASH UID文件，从默认值1开始");
            Ok(1)
        }
        Err(e) => {
            error!("读取CASH UID文件失败: {}", e);
            Err(e).with_context(|| format!("读取路径为 '{}' 的CASH UID文件失败", path))
        }
    }
}

/// 保存 Cash UID 计数器
pub fn save_uid() -> Result<()> {
    let uid = CASH_UID_COUNTER.load(Ordering::SeqCst);
    let path = "./data/cash_uid_counter";
    let mut file = File::create(path).with_context(|| format!("无法创建文件 '{}'", path))?;

    file.write_all(uid.to_string().as_bytes())
        .with_context(|| format!("写入CASH UID到文件 '{}' 失败", path))?;

    debug!("成功保存CASH UID: {} 到文件", uid);
    Ok(())
}

/// Cash 模块初始化函数
pub fn init() -> Result<()> {
    std::fs::create_dir_all("./data").with_context(|| "无法创建data目录")?;

    let saved_uid = load_saved_cash_uid().context("初始化期间加载已保存的CASH UID失败")?;
    CASH_UID_COUNTER.store(saved_uid, Ordering::SeqCst);
    info!("CASH UID计数器初始化为 {}", saved_uid);
    save_uid()?;
    Ok(())
}
