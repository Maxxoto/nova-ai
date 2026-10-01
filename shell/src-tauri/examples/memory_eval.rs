//! RFC-0011 §7 — the personal memory eval (LongMemEval-style, offline).
//!
//! Seeds a synthetic store with notes spread over time, then measures
//! recall@5 and MRR per ability group against thresholds. Run it whenever
//! the ranking (D6) or reflection (D7) changes:
//!
//! ```bash
//! cargo run --example memory_eval
//! ```
//!
//! Groups mirror LongMemEval's five abilities plus a STALE-style conflict
//! case; thresholds are the contract — a regression fails the run (exit 1).

use std::time::{SystemTime, UNIX_EPOCH};

use nova_shell::memory::{MemoryStore, MemoryType};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn days_ago(n: u64) -> String {
    nova_shell::memory::iso_utc(now_ms().saturating_sub(n * 86_400_000))
}

fn seed(store: &MemoryStore, kind: MemoryType, origin: &str, age_days: u64, body: &str) -> String {
    let mut entry = nova_shell::memory::MemoryEntry::new(kind, body.to_string());
    entry.origin = origin.to_string();
    entry.created = days_ago(age_days);
    store.write(&entry).expect("seed write");
    entry.id
}

struct Case {
    group: &'static str,
    query: &'static str,
    expect_any: Vec<String>,
}

fn main() {
    let dir = std::env::temp_dir().join(format!("ruoxi-memory-eval-{}", now_ms()));
    let store = MemoryStore::open(&dir).expect("open");

    // ---- corpus ----------------------------------------------------------
    // Keyword facts (information extraction).
    let capsaicin = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        20,
        "# Capsaicin\nCapsaicin binds the TRPV1 receptor — that is why chili feels hot.",
    );
    let zettel = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        15,
        "# Zettelkasten\nA Zettelkasten links atomic notes by idea, not by topic folder.",
    );
    // Multi-session reasoning: two halves of one fact, saved days apart.
    let travel_a = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        12,
        "# Trip part one\nThe conference flight leaves Tuesday morning.",
    );
    let travel_b = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        9,
        "# Trip part two\nThe hotel booking confirms the same Tuesday arrival.",
    );
    // Temporal: recent note only.
    let fresh = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        2,
        "# New dentist\nSwitched dentists — first appointment is this Friday.",
    );
    let _old = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        40,
        "# Old dentist\nThe old dentist was on Elm Street.",
    );
    // Knowledge update / STALE-style conflict: newer supersedes older.
    let _old_office = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        35,
        "# Office\nI work at Acme Corp on Main Street.",
    );
    let new_office = seed(
        &store,
        MemoryType::Semantic,
        "user-save",
        5,
        "# Office moved\nI moved offices — I now work at Acme Corp on Harbor Street.",
    );
    // Episodic noise (should not crowd out semantic hits).
    for day in 3..8 {
        seed(
            &store,
            MemoryType::Episodic,
            "auto",
            day,
            &format!("# Session\nAsked something unrelated number {} about tabs.", day),
        );
    }

    // ---- query set -------------------------------------------------------
    let cases = [
        Case { group: "extraction", query: "why does chili feel hot", expect_any: vec![capsaicin.clone()] },
        Case { group: "extraction", query: "how does a zettelkasten organize notes", expect_any: vec![zettel.clone()] },
        Case {
            group: "multi-session",
            query: "when is the conference trip",
            expect_any: vec![travel_a.clone(), travel_b.clone()],
        },
        Case { group: "temporal", query: "dentist last week", expect_any: vec![fresh.clone()] },
        Case { group: "knowledge-update", query: "where do I work", expect_any: vec![new_office.clone()] },
        Case { group: "knowledge-update", query: "office address", expect_any: vec![new_office.clone()] },
        Case { group: "abstention", query: "quantum entanglement homework", expect_any: vec![] },
        Case { group: "abstention", query: "cat vaccination schedule", expect_any: vec![] },
    ];

    // ---- measure ---------------------------------------------------------
    let mut failures = 0;
    let mut by_group: std::collections::BTreeMap<&str, (usize, usize, f64)> = Default::default();
    println!("memory_eval — recall@5 / MRR per ability\n");
    for case in &cases {
        let hits = store.search(case.query, None, 5).expect("search");
        let rank = hits
            .iter()
            .position(|h| case.expect_any.contains(&h.id))
            .map(|i| i + 1);
        let recall = if case.expect_any.is_empty() {
            // Abstention: surfacing anything is a failure — the block should
            // stay empty so the model says "not in memory".
            if hits.is_empty() {
                1.0
            } else {
                0.0
            }
        } else {
            rank.map(|_| 1.0).unwrap_or(0.0)
        };
        let mrr = if case.expect_any.is_empty() {
            if hits.is_empty() { 1.0 } else { 0.0 }
        } else {
            rank.map(|r| 1.0 / r as f64).unwrap_or(0.0)
        };
        let entry = by_group.entry(case.group).or_insert((0, 0, 0.0));
        entry.0 += 1;
        entry.1 += recall as usize;
        entry.2 += mrr;
        let shown = hits.first().map(|h| h.id.as_str()).unwrap_or("—");
        println!(
            "  [{:<16}] {:<38} recall={} mrr={:.2} top={}",
            case.group, case.query, recall, mrr, shown
        );
    }

    println!();
    let thresholds = [
        ("extraction", 0.99),
        ("multi-session", 0.50),
        ("temporal", 0.99),
        ("knowledge-update", 0.99),
        ("abstention", 0.99),
    ];
    for (group, threshold) in thresholds {
        if let Some((n, hits, mrr_sum)) = by_group.get(group) {
            let recall = *hits as f64 / *n as f64;
            let mrr = mrr_sum / *n as f64;
            let ok = recall >= threshold;
            if !ok {
                failures += 1;
            }
            println!(
                "  {:<16} recall@5 {:.2} (≥{:.2} {}) · mrr {:.2}",
                group,
                recall,
                threshold,
                if ok { "PASS" } else { "FAIL" },
                mrr
            );
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
    if failures > 0 {
        eprintln!("\nmemory_eval: {failures} group(s) below threshold");
        std::process::exit(1);
    }
    println!("\nmemory_eval: all groups at or above threshold");
}
