mod success;
mod failure;

fn dummy_counter() -> prometheus::IntCounterVec {
    prometheus::IntCounterVec::new(prometheus::Opts::new("fixerss_scrapes", "help"), &["feed_name"]).unwrap()
}