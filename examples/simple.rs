use chrono::Utc;
use leitner_box::{Card, LeitnerScheduler, Rating};

fn main() {
    // Define the box intervals (in days)
    let box_intervals = vec![1, 2, 7];

    // Create a Leitner scheduler with the specified box intervals and a start date
    let scheduler = LeitnerScheduler::new(box_intervals, Some(Utc::now()), "first_box").unwrap();

    // Create a new card
    let card = Card::new(Some(1), 1, None);

    // Review the card and get the updated card and review log
    let (updated_card, review_log) = scheduler
        .review_card(card, Rating::Pass, Some(Utc::now()))
        .unwrap();

    // Print the updated card and review log
    println!("Updated card: {:?}", updated_card);
    println!("Review log: {:?}", review_log);
}
