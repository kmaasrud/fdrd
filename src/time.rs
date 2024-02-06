use chrono::Duration;

pub fn format_duration(dur: Duration) -> String {
    let mut val = dur.num_seconds();
    let mut descriptor = "second";

    if dur.num_weeks() > 52 {
        descriptor = "year";
        val = dur.num_days() / 365;
    } else if dur.num_weeks() > 4 {
        descriptor = "month";
        val = dur.num_weeks() / 4;
    } else if dur.num_days() > 7 {
        descriptor = "week";
        val = dur.num_weeks();
    } else if dur.num_hours() > 24 {
        descriptor = "day";
        val = dur.num_days();
    } else if dur.num_minutes() > 60 {
        descriptor = "hour";
        val = dur.num_hours();
    } else if dur.num_seconds() > 60 {
        descriptor = "minute";
        val = dur.num_minutes();
    }

    let mut s = format!("{val} {descriptor}");
    if val != 1 {
        s.push('s');
    }
    s
}
