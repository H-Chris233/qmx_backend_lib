use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::info;
use std::sync::{Arc, RwLock};

use crate::cash::{Cash, CashDatabase, Installment};
use crate::database::Database as DbContainer;
use crate::stats::{DashboardStats, get_dashboard_stats};
use crate::student::{Class, Student, StudentDatabase, Subject};

/// QMX管理器 - 统一的API入口点
///
/// 提供线程安全的数据库操作接口，自动处理数据持久化和错误管理
pub struct QmxManager {
    database: Arc<RwLock<DbContainer>>,
    auto_save: bool,
    student_path: Option<String>,
    cash_path: Option<String>,
}

impl QmxManager {
    /// 创建新的QMX管理器实例
    ///
    /// # 参数
    /// * `auto_save` - 是否在每次操作后自动保存数据
    ///
    /// # 示例
    /// ```rust
    /// use qmx_backend_lib::QmxManager;
    ///
    /// let manager = QmxManager::new(true)?;
    /// ```
    pub fn new(auto_save: bool) -> Result<Self> {
        info!("正在初始化QMX管理器");
        let database = crate::database::init().with_context(|| "初始化数据库失败")?;

        Ok(Self {
            database: Arc::new(RwLock::new(database)),
            auto_save,
            student_path: None,
            cash_path: None,
        })
    }

    /// 从指定路径加载数据库
    pub fn from_path(student_path: &str, cash_path: &str, auto_save: bool) -> Result<Self> {
        info!(
            "从指定路径加载数据库: student={}, cash={}",
            student_path, cash_path
        );

        let student_db = StudentDatabase::read_from(student_path)
            .with_context(|| format!("无法加载学生数据库: {}", student_path))?;
        let cash_db = CashDatabase::read_from(cash_path)
            .with_context(|| format!("无法加载现金数据库: {}", cash_path))?;

        let database = DbContainer::new(student_db, cash_db);

        Ok(Self {
            database: Arc::new(RwLock::new(database)),
            auto_save,
            student_path: Some(student_path.to_string()),
            cash_path: Some(cash_path.to_string()),
        })
    }

    /// 手动保存所有数据
    pub fn save(&self) -> Result<()> {
        let db = self.database.read().unwrap();

        // 如果有自定义路径，使用自定义路径保存
        if let (Some(student_path), Some(cash_path)) = (&self.student_path, &self.cash_path) {
            info!("使用自定义路径保存数据库");
            db.student
                .save_to(student_path)
                .with_context(|| "学生数据库持久化失败")?;
            db.cash
                .save_to(cash_path)
                .with_context(|| "现金数据库持久化失败")?;
        } else {
            // 使用默认路径保存
            db.save().with_context(|| "保存数据库失败")?;
        }

        Ok(())
    }

    /// 自动保存（如果启用）
    fn auto_save_if_enabled(&self) -> Result<()> {
        if self.auto_save {
            self.save()?;
        }
        Ok(())
    }
}

// ============================================================================
// 学生管理API
// ============================================================================

impl QmxManager {
    /// 创建新学生
    ///
    /// # 参数
    /// * `builder` - 学生构建器，使用链式调用设置属性
    ///
    /// # 示例
    /// ```rust
    /// let student_id = manager.create_student(
    ///     StudentBuilder::new("张三", 16)
    ///         .phone("13800138000")
    ///         .class(Class::TenTry)
    ///         .subject(Subject::Math)
    ///         .note("优秀学生")
    /// )?;
    /// ```
    pub fn create_student(&self, builder: StudentBuilder) -> Result<u64> {
        let mut db = self.database.write().unwrap();
        let student = builder.build();
        let uid = student.uid();
        db.student.insert(student);
        drop(db);

        self.auto_save_if_enabled()?;
        info!("创建学生成功，UID: {}", uid);
        Ok(uid)
    }

    /// 获取学生信息
    pub fn get_student(&self, uid: u64) -> Result<Option<Student>> {
        let db = self.database.read().unwrap();
        Ok(db.student.get(&uid).cloned())
    }

    /// 更新学生信息
    pub fn update_student(&self, uid: u64, updater: StudentUpdater) -> Result<()> {
        let mut db = self.database.write().unwrap();
        updater.apply(&mut db.student, uid)?;
        drop(db);

        self.auto_save_if_enabled()?;
        info!("更新学生信息成功，UID: {}", uid);
        Ok(())
    }

    /// 删除学生
    pub fn delete_student(&self, uid: u64) -> Result<bool> {
        let mut db = self.database.write().unwrap();
        let removed = db.student.remove(&uid).is_some();
        drop(db);

        if removed {
            self.auto_save_if_enabled()?;
            info!("删除学生成功，UID: {}", uid);
        }
        Ok(removed)
    }

    /// 搜索学生
    pub fn search_students(&self, query: StudentQuery) -> Result<Vec<Student>> {
        let db = self.database.read().unwrap();
        Ok(query.execute(&db.student))
    }

    /// 获取所有学生
    pub fn list_students(&self) -> Result<Vec<Student>> {
        let db = self.database.read().unwrap();
        Ok(db.student.iter().map(|(_, s)| s.clone()).collect())
    }
}

// ============================================================================
// 现金管理API
// ============================================================================

impl QmxManager {
    /// 记录现金流
    pub fn record_cash(&self, builder: CashBuilder) -> Result<u64> {
        let mut db = self.database.write().unwrap();
        let cash = builder.build();
        let uid = cash.uid;
        db.cash.insert(cash);
        drop(db);

        self.auto_save_if_enabled()?;
        info!("记录现金流成功，UID: {}", uid);
        Ok(uid)
    }

    /// 获取现金记录
    pub fn get_cash(&self, uid: u64) -> Result<Option<Cash>> {
        let db = self.database.read().unwrap();
        Ok(db.cash.get(&uid).cloned())
    }

    /// 更新现金记录
    pub fn update_cash(&self, uid: u64, updater: CashUpdater) -> Result<()> {
        let mut db = self.database.write().unwrap();
        updater.apply(&mut db.cash, uid)?;
        drop(db);

        self.auto_save_if_enabled()?;
        info!("更新现金记录成功，UID: {}", uid);
        Ok(())
    }

    /// 删除现金记录
    pub fn delete_cash(&self, uid: u64) -> Result<bool> {
        let mut db = self.database.write().unwrap();
        let removed = db.cash.remove(&uid).is_some();
        drop(db);

        if removed {
            self.auto_save_if_enabled()?;
            info!("删除现金记录成功，UID: {}", uid);
        }
        Ok(removed)
    }

    /// 搜索现金记录
    pub fn search_cash(&self, query: CashQuery) -> Result<Vec<Cash>> {
        let db = self.database.read().unwrap();
        Ok(query.execute(&db.cash))
    }

    /// 获取学生的所有现金记录
    pub fn get_student_cash(&self, student_id: u64) -> Result<Vec<Cash>> {
        let db = self.database.read().unwrap();
        Ok(db
            .cash
            .iter()
            .filter_map(|(_, c)| {
                if c.student_id == Some(student_id) {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect())
    }
}

// ============================================================================
// 统计分析API
// ============================================================================

impl QmxManager {
    /// 获取仪表板统计信息
    pub fn get_dashboard_stats(&self) -> Result<DashboardStats> {
        let db = self.database.read().unwrap();
        get_dashboard_stats(&db.student, &db.cash).with_context(|| "获取统计信息失败")
    }

    /// 获取学生统计信息
    pub fn get_student_stats(&self, uid: u64) -> Result<StudentStats> {
        let db = self.database.read().unwrap();
        StudentStats::calculate(&db.student, &db.cash, uid)
    }

    /// 获取财务统计信息
    pub fn get_financial_stats(&self, period: TimePeriod) -> Result<FinancialStats> {
        let db = self.database.read().unwrap();
        FinancialStats::calculate(&db.cash, period)
    }
}

// ============================================================================
// 构建器模式
// ============================================================================

/// 学生构建器 - 使用构建器模式创建学生
pub struct StudentBuilder {
    name: String,
    age: u8,
    phone: Option<String>,
    class: Option<Class>,
    subject: Option<Subject>,
    lesson_left: Option<u32>,
    note: Option<String>,
    membership_start: Option<DateTime<Utc>>,
    membership_end: Option<DateTime<Utc>>,
}

impl StudentBuilder {
    pub fn new(name: impl Into<String>, age: u8) -> Self {
        Self {
            name: name.into(),
            age,
            phone: None,
            class: None,
            subject: None,
            lesson_left: None,
            note: None,
            membership_start: None,
            membership_end: None,
        }
    }

    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn class(mut self, class: Class) -> Self {
        self.class = Some(class);
        self
    }

    pub fn subject(mut self, subject: Subject) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn lesson_left(mut self, lessons: u32) -> Self {
        self.lesson_left = Some(lessons);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn membership(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.membership_start = Some(start);
        self.membership_end = Some(end);
        self
    }

    fn build(self) -> Student {
        let mut s = Student::new();
        s.set_name(self.name);
        s.set_age(self.age);
        if let Some(phone) = self.phone {
            s.set_phone(phone);
        }
        if let Some(class) = self.class {
            s.set_class(class);
        }
        if let Some(subject) = self.subject {
            s.set_subject(subject);
        }
        if let Some(lesson) = self.lesson_left {
            s.set_lesson_left(lesson);
        }
        if let Some(note) = self.note {
            s.set_note(note);
        }
        if self.membership_start.is_some() || self.membership_end.is_some() {
            s.set_membership_dates(self.membership_start, self.membership_end);
        }
        s
    }
}

/// 现金构建器
pub struct CashBuilder {
    student_id: Option<u64>,
    amount: i64,
    note: Option<String>,
    installment: Option<Installment>,
}

impl CashBuilder {
    pub fn new(amount: i64) -> Self {
        Self {
            student_id: None,
            amount,
            note: None,
            installment: None,
        }
    }

    pub fn student_id(mut self, student_id: u64) -> Self {
        self.student_id = Some(student_id);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn installment(mut self, installment: Installment) -> Self {
        self.installment = Some(installment);
        self
    }

    fn build(self) -> Cash {
        let mut c = Cash::new(self.student_id);
        c.set_cash(self.amount);
        if let Some(n) = self.note {
            c.set_note(Some(n));
        }
        if let Some(inst) = self.installment {
            c.installment = Some(inst);
        }
        c
    }
}

// ============================================================================
// 更新器模式
// ============================================================================

/// 学生更新器 - 用于更新现有学生信息
pub struct StudentUpdater {
    updates: Vec<StudentUpdate>,
}

enum StudentUpdate {
    Name(String),
    Age(u8),
    Phone(String),
    Class(Class),
    Subject(Subject),
    LessonLeft(Option<u32>),
    Note(String),
    AddRing(f64),
    SetRings(Vec<f64>),
    Membership(Option<DateTime<Utc>>, Option<DateTime<Utc>>),
}

impl StudentUpdater {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.updates.push(StudentUpdate::Name(name.into()));
        self
    }

    pub fn age(mut self, age: u8) -> Self {
        self.updates.push(StudentUpdate::Age(age));
        self
    }

    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.updates.push(StudentUpdate::Phone(phone.into()));
        self
    }

    pub fn class(mut self, class: Class) -> Self {
        self.updates.push(StudentUpdate::Class(class));
        self
    }

    pub fn subject(mut self, subject: Subject) -> Self {
        self.updates.push(StudentUpdate::Subject(subject));
        self
    }

    pub fn lesson_left(mut self, lessons: Option<u32>) -> Self {
        self.updates.push(StudentUpdate::LessonLeft(lessons));
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.updates.push(StudentUpdate::Note(note.into()));
        self
    }

    pub fn add_ring(mut self, score: f64) -> Self {
        self.updates.push(StudentUpdate::AddRing(score));
        self
    }

    pub fn set_rings(mut self, rings: Vec<f64>) -> Self {
        self.updates.push(StudentUpdate::SetRings(rings));
        self
    }

    pub fn membership(mut self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        self.updates.push(StudentUpdate::Membership(start, end));
        self
    }

    fn apply(self, db: &mut StudentDatabase, uid: u64) -> Result<()> {
        let student = db
            .student_data
            .get_mut(&uid)
            .ok_or_else(|| anyhow::anyhow!("学生不存在: {}", uid))?;

        for update in self.updates {
            match update {
                StudentUpdate::Name(name) => {
                    student.set_name(name);
                }
                StudentUpdate::Age(age) => {
                    student.set_age(age);
                }
                StudentUpdate::Phone(phone) => {
                    student.set_phone(phone);
                }
                StudentUpdate::Class(class) => {
                    student.set_class(class);
                }
                StudentUpdate::Subject(subject) => {
                    student.set_subject(subject);
                }
                StudentUpdate::LessonLeft(lessons) => {
                    if let Some(v) = lessons {
                        student.set_lesson_left(v);
                    }
                }
                StudentUpdate::Note(note) => {
                    student.set_note(note);
                }
                StudentUpdate::AddRing(score) => {
                    student.add_ring(score);
                }
                StudentUpdate::SetRings(rings) => {
                    for r in rings {
                        student.add_ring(r);
                    }
                }
                StudentUpdate::Membership(start, end) => {
                    student.set_membership_dates(start, end);
                }
            }
        }

        Ok(())
    }
}

/// 现金更新器
pub struct CashUpdater {
    updates: Vec<CashUpdate>,
}

enum CashUpdate {
    StudentId(Option<u64>),
    Amount(i64),
    Note(Option<String>),
    Installment(Option<Installment>),
}

impl CashUpdater {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub fn student_id(mut self, student_id: Option<u64>) -> Self {
        self.updates.push(CashUpdate::StudentId(student_id));
        self
    }

    pub fn amount(mut self, amount: i64) -> Self {
        self.updates.push(CashUpdate::Amount(amount));
        self
    }

    pub fn note(mut self, note: Option<String>) -> Self {
        self.updates.push(CashUpdate::Note(note));
        self
    }

    pub fn installment(mut self, installment: Option<Installment>) -> Self {
        self.updates.push(CashUpdate::Installment(installment));
        self
    }

    fn apply(self, db: &mut CashDatabase, uid: u64) -> Result<()> {
        let cash = db
            .cash_data
            .get_mut(&uid)
            .ok_or_else(|| anyhow::anyhow!("现金记录不存在: {}", uid))?;

        for update in self.updates {
            match update {
                CashUpdate::StudentId(student_id) => cash.student_id = student_id,
                CashUpdate::Amount(amount) => cash.cash = amount,
                CashUpdate::Note(note) => cash.note = note,
                CashUpdate::Installment(installment) => cash.installment = installment,
            }
        }

        Ok(())
    }
}

// ============================================================================
// 查询构建器
// ============================================================================

/// 学生查询构建器
pub struct StudentQuery {
    filters: Vec<StudentFilter>,
}

enum StudentFilter {
    Name(String),
    AgeRange(u8, u8),
    Class(Class),
    Subject(Subject),
    HasMembership(bool),
    MembershipActive(DateTime<Utc>),
}

impl StudentQuery {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn name_contains(mut self, name: impl Into<String>) -> Self {
        self.filters.push(StudentFilter::Name(name.into()));
        self
    }

    pub fn age_range(mut self, min: u8, max: u8) -> Self {
        self.filters.push(StudentFilter::AgeRange(min, max));
        self
    }

    pub fn class(mut self, class: Class) -> Self {
        self.filters.push(StudentFilter::Class(class));
        self
    }

    pub fn subject(mut self, subject: Subject) -> Self {
        self.filters.push(StudentFilter::Subject(subject));
        self
    }

    pub fn has_membership(mut self, has: bool) -> Self {
        self.filters.push(StudentFilter::HasMembership(has));
        self
    }

    pub fn membership_active_at(mut self, date: DateTime<Utc>) -> Self {
        self.filters.push(StudentFilter::MembershipActive(date));
        self
    }

    fn execute(self, db: &StudentDatabase) -> Vec<Student> {
        db.iter()
            .map(|(_, s)| s.clone())
            .into_iter()
            .filter(|student| {
                self.filters.iter().all(|filter| match filter {
                    StudentFilter::Name(name) => student.name().contains(name),
                    StudentFilter::AgeRange(min, max) => {
                        student.age() >= *min && student.age() <= *max
                    }
                    StudentFilter::Class(class) => student.class() == class,
                    StudentFilter::Subject(subject) => student.subject() == subject,
                    StudentFilter::HasMembership(has) => {
                        student.membership_start_date().is_some() == *has
                    }
                    StudentFilter::MembershipActive(date) => {
                        if let (Some(start), Some(end)) = (
                            student.membership_start_date(),
                            student.membership_end_date(),
                        ) {
                            *date >= start && *date <= end
                        } else {
                            false
                        }
                    }
                })
            })
            .collect()
    }
}

/// 现金查询构建器
pub struct CashQuery {
    filters: Vec<CashFilter>,
}

enum CashFilter {
    StudentId(u64),
    AmountRange(i64, i64),
    HasInstallment(bool),
    DateRange(DateTime<Utc>, DateTime<Utc>),
}

impl CashQuery {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn student_id(mut self, student_id: u64) -> Self {
        self.filters.push(CashFilter::StudentId(student_id));
        self
    }

    pub fn amount_range(mut self, min: i64, max: i64) -> Self {
        self.filters.push(CashFilter::AmountRange(min, max));
        self
    }

    pub fn has_installment(mut self, has: bool) -> Self {
        self.filters.push(CashFilter::HasInstallment(has));
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filters.push(CashFilter::DateRange(start, end));
        self
    }

    fn execute(self, db: &CashDatabase) -> Vec<Cash> {
        db.iter()
            .map(|(_, c)| c.clone())
            .into_iter()
            .filter(|cash| {
                self.filters.iter().all(|filter| match filter {
                    CashFilter::StudentId(id) => cash.student_id == Some(*id),
                    CashFilter::AmountRange(min, max) => cash.cash >= *min && cash.cash <= *max,
                    CashFilter::HasInstallment(has) => cash.installment.is_some() == *has,
                    CashFilter::DateRange(start, end) => {
                        cash.created_at >= *start && cash.created_at <= *end
                    }
                })
            })
            .collect()
    }
}

// ============================================================================
// 统计类型
// ============================================================================

/// 学生统计信息
#[derive(Debug, Clone)]
pub struct StudentStats {
    pub total_payments: i64,
    pub payment_count: usize,
    pub average_score: Option<f64>,
    pub score_count: usize,
    pub membership_status: MembershipStatus,
}

/// 会员状态
#[derive(Debug, Clone)]
pub enum MembershipStatus {
    None,
    Active { expires_at: DateTime<Utc> },
    Expired { expired_at: DateTime<Utc> },
}

impl StudentStats {
    fn calculate(student_db: &StudentDatabase, cash_db: &CashDatabase, uid: u64) -> Result<Self> {
        let student = student_db
            .get(&uid)
            .ok_or_else(|| anyhow::anyhow!("学生不存在: {}", uid))?;

        let cash_records: Vec<_> = cash_db
            .iter()
            .filter_map(|(_, c)| {
                if c.student_id == Some(uid) {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        let total_payments: i64 = cash_records.iter().map(|c| c.cash).sum();
        let payment_count = cash_records.len();

        let rings = student.rings();
        let average_score = if rings.is_empty() {
            None
        } else {
            Some(rings.iter().sum::<f64>() / rings.len() as f64)
        };

        let membership_status = match (
            student.membership_start_date(),
            student.membership_end_date(),
        ) {
            (Some(_start), Some(end)) => {
                let now = Utc::now();
                if now <= end {
                    MembershipStatus::Active { expires_at: end }
                } else {
                    MembershipStatus::Expired { expired_at: end }
                }
            }
            _ => MembershipStatus::None,
        };

        Ok(Self {
            total_payments,
            payment_count,
            average_score,
            score_count: rings.len(),
            membership_status,
        })
    }
}

/// 财务统计信息
#[derive(Debug, Clone)]
pub struct FinancialStats {
    pub total_income: i64,
    pub total_expense: i64,
    pub net_income: i64,
    pub transaction_count: usize,
    pub installment_count: usize,
}

/// 时间周期
#[derive(Debug, Clone)]
pub enum TimePeriod {
    Today,
    ThisWeek,
    ThisMonth,
    ThisYear,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

impl FinancialStats {
    fn calculate(cash_db: &CashDatabase, _period: TimePeriod) -> Result<Self> {
        let all_cash: Vec<_> = cash_db.iter().map(|(_, c)| c.clone()).collect();

        let total_income: i64 = all_cash.iter().filter(|c| c.cash > 0).map(|c| c.cash).sum();
        let total_expense: i64 = all_cash
            .iter()
            .filter(|c| c.cash < 0)
            .map(|c| c.cash.abs())
            .sum();
        let net_income = total_income - total_expense;
        let transaction_count = all_cash.len();
        let installment_count = all_cash.iter().filter(|c| c.installment.is_some()).count();

        Ok(Self {
            total_income,
            total_expense,
            net_income,
            transaction_count,
            installment_count,
        })
    }
}
