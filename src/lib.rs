#![deny(warnings)]

use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Rating {
    Fail = 0,
    Pass = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub card_id: i64,
    pub box_num: i32,
    pub due: Option<DateTime<Utc>>,
}

impl Card {
    pub fn new(card_id: Option<i64>, box_num: i32, due: Option<DateTime<Utc>>) -> Self {
        let card_id = card_id.unwrap_or_else(|| Utc::now().timestamp_millis());
        Self {
            card_id,
            box_num,
            due,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub card: Card,
    pub rating: Rating,
    pub review_datetime: DateTime<Utc>,
}

impl ReviewLog {
    pub const fn new(card: Card, rating: Rating, review_datetime: DateTime<Utc>) -> Self {
        Self {
            card,
            rating,
            review_datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeitnerScheduler {
    pub box_intervals: Vec<i32>,
    pub start_datetime: NaiveDateTime,
    pub on_fail: String,
}

impl LeitnerScheduler {
    pub fn new(
        box_intervals: Vec<i32>,
        start_datetime: Option<DateTime<Utc>>,
        on_fail: &str,
    ) -> Result<Self, &'static str> {
        if box_intervals[0] != 1 {
            return Err("Box 1 must have an interval of 1 day.");
        }

        let start_datetime = start_datetime.unwrap_or_else(Utc::now).naive_utc();

        Ok(Self {
            box_intervals,
            start_datetime,
            on_fail: on_fail.to_string(),
        })
    }

    pub fn review_card(
        &self,
        mut card: Card,
        rating: Rating,
        review_datetime: Option<DateTime<Utc>>,
    ) -> Result<(Card, ReviewLog), &'static str> {
        let review_datetime = review_datetime.unwrap_or_else(Utc::now);

        let review_log = ReviewLog::new(card.clone(), rating, review_datetime);

        if card.due.is_none() {
            card.due = review_datetime
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .map(|x| Utc.from_utc_datetime(&x));
        }

        if review_datetime < card.due.unwrap() {
            return Err("Card is not due for review yet.");
        }

        match rating {
            Rating::Fail => {
                if self.on_fail == "first_box" {
                    card.box_num = 1;
                } else if self.on_fail == "prev_box" && card.box_num > 1 {
                    card.box_num -= 1;
                }
            }
            Rating::Pass => {
                if card.box_num < self.box_intervals.len() as i32 {
                    card.box_num += 1;
                }
            }
        }

        let interval = self.box_intervals[(card.box_num - 1) as usize];

        let begin_datetime = self.start_datetime - Duration::days(1);
        let mut i = 1;
        let mut next_due_date = begin_datetime + Duration::days(interval as i64);

        while next_due_date <= review_datetime.naive_utc() {
            next_due_date = begin_datetime + Duration::days((interval * i) as i64);
            i += 1;
        }

        card.due = Some(Utc.from_utc_datetime(&next_due_date));

        Ok((card, review_log))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_card_pass() {
        let scheduler =
            LeitnerScheduler::new(vec![1, 2, 7], Some(Utc::now()), "first_box").unwrap();
        let card = Card::new(Some(1), 1, None);

        let review_datetime = Utc::now();

        let (updated_card, _) = scheduler
            .review_card(card, Rating::Pass, Some(review_datetime))
            .unwrap();
        assert_eq!(updated_card.box_num, 2);
    }
}
