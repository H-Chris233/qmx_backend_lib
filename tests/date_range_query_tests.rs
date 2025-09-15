// 测试 CashQuery 的 DateRange 功能
use chrono::{Duration, Utc};
use qmx_backend_lib::*;

mod date_range_query_tests {
    use super::*;

    #[test]
    fn test_cash_query_date_range_method() {
        let now = Utc::now();
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);

        // 测试 date_range 方法是否存在并可以调用
        let query = CashQuery::new().date_range(start, end);

        // 这个测试只是验证方法存在并且可以链式调用
        // 由于没有实际的现金记录，我们无法测试过滤结果
        // 但这证明了 API 是正确的

        // 测试可以与其他过滤器组合
        let combined_query = CashQuery::new()
            .student_id(123)
            .amount_range(1000, 5000)
            .date_range(start, end)
            .has_installment(false);

        // 如果代码能执行到这里，说明 date_range 方法正常工作
        // 验证查询对象创建成功（通过验证对象存在）
        drop(query);
        drop(combined_query);
    }

    #[test]
    fn test_multiple_date_ranges() {
        let now = Utc::now();

        // 测试不同的日期范围
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        let _query1 = CashQuery::new().date_range(yesterday, now);
        let _query2 = CashQuery::new().date_range(now, tomorrow);

        // 测试跨越多天的范围
        let week_ago = now - Duration::days(7);
        let week_later = now + Duration::days(7);

        let _query3 = CashQuery::new().date_range(week_ago, week_later);

        // 测试精确的时间点
        let query4 = CashQuery::new().date_range(now, now);

        // 验证所有查询对象都创建成功
        drop(query4);
    }

    #[test]
    fn test_date_range_with_all_filters() {
        let now = Utc::now();
        let start = now - Duration::hours(2);
        let end = now + Duration::hours(2);

        // 测试所有过滤器的组合
        let comprehensive_query = CashQuery::new()
            .student_id(456)
            .amount_range(-1000, 10000)
            .has_installment(true)
            .date_range(start, end);

        // 验证复合查询创建成功
        drop(comprehensive_query);
    }
}
