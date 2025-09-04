use crate::models::{ProjectTimeBreakdown, TimeRecord};
use chrono::{DateTime, Datelike, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct TimeCalculator;

impl TimeCalculator {
    /// 计算指定时间范围内的项目内时间
    pub fn calculate_project_time(
        time_records: &[&TimeRecord],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> i64 {
        time_records
            .iter()
            .filter(|record| {
                record.project_id.is_some()
                    && record.start_time >= start_time
                    && record.start_time <= end_time
            })
            .map(|record| record.duration_minutes)
            .sum()
    }

    /// 计算指定时间范围内的项目外时间
    pub fn calculate_non_project_time(
        time_records: &[&TimeRecord],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> i64 {
        time_records
            .iter()
            .filter(|record| {
                record.project_id.is_none()
                    && record.start_time >= start_time
                    && record.start_time <= end_time
            })
            .map(|record| record.duration_minutes)
            .sum()
    }

    /// 计算指定项目的总时间
    pub fn calculate_project_total_time(
        time_records: &[&TimeRecord],
        project_id: Uuid,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> i64 {
        time_records
            .iter()
            .filter(|record| {
                record.project_id == Some(project_id)
                    && start_time.map_or(true, |start| record.start_time >= start)
                    && end_time.map_or(true, |end| record.start_time <= end)
            })
            .map(|record| record.duration_minutes)
            .sum()
    }

    /// 生成项目时间分解
    pub fn generate_project_breakdown(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<ProjectTimeBreakdown> {
        let mut project_times: HashMap<Uuid, (i64, i32)> = HashMap::new();

        // 统计每个项目的总时间和事件数量
        for record in time_records {
            if record.project_id.is_some()
                && record.start_time >= start_time
                && record.start_time <= end_time
            {
                let project_id = record.project_id.unwrap();
                let entry = project_times.entry(project_id).or_insert((0, 0));
                entry.0 += record.duration_minutes;
                entry.1 += 1;
            }
        }

        // 创建项目时间分解结构
        project_times
            .into_iter()
            .map(
                |(project_id, (total_time, event_count))| ProjectTimeBreakdown {
                    project_id,
                    project_name: project_names
                        .get(&project_id)
                        .cloned()
                        .unwrap_or_else(|| "未知项目".to_string()),
                    total_time_minutes: total_time,
                    event_count,
                },
            )
            .collect()
    }

    /// 获取一周的开始时间（周一）
    pub fn get_week_start(date: DateTime<Utc>) -> DateTime<Utc> {
        let days_since_monday = date.weekday().num_days_from_monday();
        date - chrono::Duration::days(days_since_monday as i64)
    }

    /// 获取一周的结束时间（周日）
    pub fn get_week_end(date: DateTime<Utc>) -> DateTime<Utc> {
        let days_until_sunday = 6 - date.weekday().num_days_from_monday();
        date + chrono::Duration::days(days_until_sunday as i64)
    }

    /// 获取指定日期所在周的所有时间记录
    pub fn get_week_time_records<'a>(
        time_records: &'a [&TimeRecord],
        date: DateTime<Utc>,
    ) -> Vec<&'a TimeRecord> {
        let week_start = Self::get_week_start(date);
        let week_end = Self::get_week_end(date);

        time_records
            .iter()
            .filter(|record| record.start_time >= week_start && record.start_time <= week_end)
            .copied()
            .collect()
    }

    /// 计算每日时间统计
    pub fn calculate_daily_stats(time_records: &[&TimeRecord], date: DateTime<Utc>) -> (i64, i64) {
        let day_start = date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

        let project_time = Self::calculate_project_time(time_records, day_start, day_end);
        let non_project_time = Self::calculate_non_project_time(time_records, day_start, day_end);

        (project_time, non_project_time)
    }

    /// 计算每周时间统计
    pub fn calculate_weekly_stats(time_records: &[&TimeRecord], date: DateTime<Utc>) -> (i64, i64) {
        let week_start = Self::get_week_start(date);
        let week_end = Self::get_week_end(date);

        let project_time = Self::calculate_project_time(time_records, week_start, week_end);
        let non_project_time = Self::calculate_non_project_time(time_records, week_start, week_end);

        (project_time, non_project_time)
    }

    /// 计算每月时间统计
    pub fn calculate_monthly_stats(
        time_records: &[&TimeRecord],
        year: i32,
        month: u32,
    ) -> (i64, i64) {
        let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let next_month = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let month_end = chrono::NaiveDate::from_ymd_opt(next_month.0, next_month.1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            - chrono::Duration::seconds(1);

        let project_time = Self::calculate_project_time(time_records, month_start, month_end);
        let non_project_time =
            Self::calculate_non_project_time(time_records, month_start, month_end);

        (project_time, non_project_time)
    }

    /// 格式化分钟数为可读格式
    pub fn format_duration(minutes: i64) -> String {
        if minutes < 60 {
            format!("{}分钟", minutes)
        } else if minutes < 1440 {
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            if remaining_minutes == 0 {
                format!("{}小时", hours)
            } else {
                format!("{}小时{}分钟", hours, remaining_minutes)
            }
        } else {
            let days = minutes / 1440;
            let remaining_hours = (minutes % 1440) / 60;
            let remaining_minutes = minutes % 60;

            let mut result = format!("{}天", days);
            if remaining_hours > 0 {
                result.push_str(&format!("{}小时", remaining_hours));
            }
            if remaining_minutes > 0 {
                result.push_str(&format!("{}分钟", remaining_minutes));
            }
            result
        }
    }

    /// 获取时间效率统计
    pub fn get_efficiency_stats(
        time_records: &[&TimeRecord],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> f64 {
        let project_time = Self::calculate_project_time(time_records, start_time, end_time);
        let non_project_time = Self::calculate_non_project_time(time_records, start_time, end_time);
        let total_time = project_time + non_project_time;

        if total_time == 0 {
            0.0
        } else {
            (project_time as f64 / total_time as f64) * 100.0
        }
    }

    /// 获取项目排名（按时间从多到少）
    pub fn get_project_ranking(
        time_records: &[&TimeRecord],
        project_names: &HashMap<Uuid, String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<(String, i64)> {
        let mut breakdown =
            Self::generate_project_breakdown(time_records, project_names, start_time, end_time);

        // 按时间降序排序
        breakdown.sort_by(|a, b| b.total_time_minutes.cmp(&a.total_time_minutes));

        breakdown
            .into_iter()
            .map(|item| (item.project_name, item.total_time_minutes))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_time_record(
        project_id: Option<Uuid>,
        start_time: DateTime<Utc>,
        duration_minutes: i64,
    ) -> TimeRecord {
        let end_time = start_time + Duration::minutes(duration_minutes);
        TimeRecord::new(Uuid::new_v4(), project_id, start_time, end_time)
    }

    #[test]
    fn test_calculate_project_time() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record1 = create_test_time_record(Some(project_id), base_time, 60);
        let record2 = create_test_time_record(Some(project_id), base_time + Duration::hours(2), 30);
        let record3 = create_test_time_record(None, base_time, 45); // 项目外时间
        let records = vec![&record1, &record2, &record3];

        let project_time = TimeCalculator::calculate_project_time(
            &records,
            base_time - Duration::hours(1),
            base_time + Duration::hours(4),
        );

        assert_eq!(project_time, 90); // 60 + 30 分钟
    }

    #[test]
    fn test_calculate_non_project_time() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record1 = create_test_time_record(Some(project_id), base_time, 60);
        let record2 = create_test_time_record(None, base_time + Duration::hours(2), 45);
        let record3 = create_test_time_record(None, base_time + Duration::hours(3), 30);
        let records = vec![&record1, &record2, &record3];

        let non_project_time = TimeCalculator::calculate_non_project_time(
            &records,
            base_time - Duration::hours(1),
            base_time + Duration::hours(4),
        );

        assert_eq!(non_project_time, 75); // 45 + 30 分钟
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(TimeCalculator::format_duration(30), "30分钟");
        assert_eq!(TimeCalculator::format_duration(90), "1小时30分钟");
        assert_eq!(TimeCalculator::format_duration(120), "2小时");
        assert_eq!(TimeCalculator::format_duration(1500), "1天1小时");
        assert_eq!(TimeCalculator::format_duration(2880), "2天");
    }

    #[test]
    fn test_week_boundaries() {
        let test_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 10) // 2024年1月10日是周三
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        let week_start = TimeCalculator::get_week_start(test_date);
        let week_end = TimeCalculator::get_week_end(test_date);

        // 周一应该是1月8日
        assert_eq!(week_start.date_naive().day(), 8);
        assert_eq!(week_start.weekday(), Weekday::Mon);

        // 周日应该是1月14日
        assert_eq!(week_end.date_naive().day(), 14);
        assert_eq!(week_end.weekday(), Weekday::Sun);
    }

    #[test]
    fn test_efficiency_stats() {
        let project_id = Uuid::new_v4();
        let base_time = Utc::now();

        let record1 = create_test_time_record(Some(project_id), base_time, 60); // 项目时间
        let record2 = create_test_time_record(None, base_time + Duration::hours(2), 30); // 非项目时间
        let records = vec![&record1, &record2];

        let efficiency = TimeCalculator::get_efficiency_stats(
            &records,
            base_time - Duration::hours(1),
            base_time + Duration::hours(3),
        );

        // 项目时间60分钟，总时间90分钟，效率应该是66.67%
        assert!((efficiency - 66.67).abs() < 0.01);
    }
}
