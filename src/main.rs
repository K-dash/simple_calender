use chrono::{NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Schedule {
    id: u64,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Schedule {
    fn intersects(&self, other: &Schedule) -> bool {
        self.start < other.end
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Calendar {
    schedules: Vec<Schedule>,
}

const SCHEDULE_FILE: &str = "schedules.json";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 予定の一覧表示
    List,
    /// 予定の追加
    Add {
        /// タイトル
        subject: String,
        /// 開始日時
        start: NaiveDateTime,
        /// 終了日時
        end: NaiveDateTime,
    },
}

fn main() {
    let options = Cli::parse();
    match options.command {
        Commands::List => show_list(),
        Commands::Add {
            subject,
            start,
            end,
        } => add_schedule(subject, start, end),
    }
}

fn show_list() {
    let file: Calendar = {
        let file = File::open(SCHEDULE_FILE).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    // 予定の表示
    println!("ID\tStart\tEnd\tSubject");
    for schedule in file.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id, schedule.start, schedule.end, schedule.subject
        );
    }
}

fn add_schedule(subject: String, start: NaiveDateTime, end: NaiveDateTime) {
    let mut calendar: Calendar = {
        let file = File::open(SCHEDULE_FILE).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };

    // 予定の作成
    let id = calendar.schedules.len() as u64;
    let new_schedule = Schedule {
        id,
        subject,
        start,
        end,
    };

    // 予定の重複判定
    for schedule in &calendar.schedules {
        if schedule.intersects(&new_schedule) {
            println!("エラー：予定が重複しています");
            return;
        }
    }

    // 予定の追加
    calendar.schedules.push(new_schedule);

    // 予定の保存
    {
        let file = File::create(SCHEDULE_FILE).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &calendar).unwrap();
    }
    println!("予定を追加しました");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    #[test]
    fn test_schedule_intersects_1() {
        // 2024年1月1日の18:15から19:15までの既存予定
        let schedule = Schedule {
            id: 1,
            subject: "既存予定1".to_string(),
            start: native_date_time(2024, 1, 1, 18, 15, 0),
            end: native_date_time(2024, 1, 1, 19, 15, 0),
        };
        // 2024年1月1日の19:00から20:00までの新規予定
        let new_schedule = Schedule {
            id: 2,
            subject: "新規予定1".to_string(),
            start: native_date_time(2024, 1, 1, 19, 0, 0),
            end: native_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert!(schedule.intersects(&new_schedule));
    }

    #[test]
    // 既存予定: 2024年1月1日の19:45から20:45まで
    // 新規予定: 2024年1月1日の19:00から20:00まで
    fn test_schedule_intersects_2() {
        let schedule = Schedule {
            id: 1,
            subject: "既存予定2".to_string(),
            start: native_date_time(2024, 1, 1, 19, 45, 0),
            end: native_date_time(2024, 1, 1, 20, 45, 0),
        };
        let new_schedule = Schedule {
            id: 2,
            subject: "新規予定2".to_string(),
            start: native_date_time(2024, 1, 1, 19, 0, 0),
            end: native_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert!(schedule.intersects(&new_schedule));
    }

    #[test]
    // 既存予定: 2024年1月1日の18:30から20:15まで
    // 新規予定: 2024年1月1日の19:00から20:00まで
    fn test_schedule_intersects_3() {
        let schedule = Schedule {
            id: 1,
            subject: "既存予定3".to_string(),
            start: native_date_time(2024, 1, 1, 18, 30, 0),
            end: native_date_time(2024, 1, 1, 20, 15, 0),
        };
        let new_schedule = Schedule {
            id: 2,
            subject: "新規予定3".to_string(),
            start: native_date_time(2024, 1, 1, 19, 0, 0),
            end: native_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert!(schedule.intersects(&new_schedule));
    }

    #[test]
    // 既存予定: 2024年1月1日の20:15から20:45まで
    // 新規予定: 2024年1月1日の19:00から20:00まで
    fn test_schedule_intersects_4() {
        let schedule = Schedule {
            id: 1,
            subject: "既存予定4".to_string(),
            start: native_date_time(2024, 1, 1, 20, 15, 0),
            end: native_date_time(2024, 1, 1, 20, 45, 0),
        };
        let new_schedule = Schedule {
            id: 2,
            subject: "新規予定4".to_string(),
            start: native_date_time(2024, 1, 1, 19, 0, 0),
            end: native_date_time(2024, 1, 1, 20, 0, 0),
        };
        assert!(!schedule.intersects(&new_schedule));
    }

    #[test]
    // 既存予定: 2024年12月8日の09:00から10:30まで
    // 新規予定: 2024年12月15日の10:00から11:00まで
    fn test_schedule_intersects_5() {
        let schedule = Schedule {
            id: 1,
            subject: "既存予定5".to_string(),
            start: native_date_time(2024, 12, 8, 9, 0, 0),
            end: native_date_time(2024, 12, 8, 10, 30, 0),
        };
        let new_schedule = Schedule {
            id: 2,
            subject: "新規予定5".to_string(),
            start: native_date_time(2024, 12, 15, 10, 0, 0),
            end: native_date_time(2024, 12, 15, 11, 0, 0),
        };
        assert!(!schedule.intersects(&new_schedule));
    }
}
