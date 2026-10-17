//! Dates as sites print them, as epoch milliseconds in UTC (what `Chapter::date_upload` wants).

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The civil date `(year, month, day)` of a day count since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    days_from_civil(if m == 12 { y + 1 } else { y }, if m == 12 { 1 } else { m + 1 }, 1) - days_from_civil(y, m, 1)
}

/// What "3 days ago" counts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// `now_ms` minus `amount` of `unit`, the way a calendar does it: months and years keep the day of the month, or the
/// last day of a shorter month.
pub fn minus(now_ms: i64, amount: i64, unit: Unit) -> i64 {
    let fixed = |secs: i64| now_ms - secs * 1000;
    match unit {
        Unit::Second => fixed(amount),
        Unit::Minute => fixed(amount * 60),
        Unit::Hour => fixed(amount * 3600),
        Unit::Day => fixed(amount * 86400),
        Unit::Week => fixed(amount * 7 * 86400),
        Unit::Month | Unit::Year => {
            let months = if unit == Unit::Year { amount * 12 } else { amount };
            let (days, ms_of_day) = (now_ms.div_euclid(86_400_000), now_ms.rem_euclid(86_400_000));
            let (y, m, d) = civil_from_days(days);
            let index = y * 12 + (m - 1) - months;
            let (ny, nm) = (index.div_euclid(12), index.rem_euclid(12) + 1);
            days_from_civil(ny, nm, d.min(days_in_month(ny, nm))) * 86_400_000 + ms_of_day
        }
    }
}

/// Epoch milliseconds of a UTC date and time.
pub fn millis(y: i64, mo: i64, d: i64, h: i64, mi: i64, s: i64) -> i64 {
    (days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + s) * 1000
}

/// ISO-8601: `2024-03-05`, `2024-03-05T10:20:30`, with optional fraction and `Z` or `+hh:mm` / `+hhmm` offset.
pub fn parse_iso(s: &str) -> Option<i64> {
    let s = s.trim();
    let num = |r: std::ops::Range<usize>| s.get(r).and_then(|p| p.parse::<i64>().ok());
    let (y, mo, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    if s.as_bytes().get(4) != Some(&b'-') || !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    let (mut h, mut mi, mut sec) = (0, 0, 0);
    let mut offset = 0;
    if matches!(s.as_bytes().get(10), Some(b'T' | b't' | b' ')) {
        h = num(11..13)?;
        mi = num(14..16)?;
        sec = num(17..19).unwrap_or(0);
        let mut rest = s.get(19..).unwrap_or("");
        if let Some(r) = rest.strip_prefix('.') {
            rest = r.trim_start_matches(|c: char| c.is_ascii_digit());
        }
        if let Some(sign) = rest.chars().next().filter(|c| matches!(c, '+' | '-')) {
            let digits: String = rest[1..].chars().filter(char::is_ascii_digit).collect();
            let oh: i64 = digits.get(0..2).and_then(|p| p.parse().ok()).unwrap_or(0);
            let om: i64 = digits.get(2..4).and_then(|p| p.parse().ok()).unwrap_or(0);
            offset = (if sign == '-' { -1 } else { 1 }) * (oh * 3600 + om * 60);
        }
    }
    Some(millis(y, mo, d, h, mi, sec) - offset * 1000)
}

/// Month names per language (full names; the first three letters serve as the short form).
fn month_names(lang: &str) -> &'static [&'static str; 12] {
    match lang.split(['-', '_']).next().unwrap_or("en") {
        "pt" => &["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"],
        "es" => &["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"],
        "fr" => &["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"],
        "id" => &["januari", "februari", "maret", "april", "mei", "juni", "juli", "agustus", "september", "oktober", "november", "desember"],
        "tr" => &["ocak", "şubat", "mart", "nisan", "mayıs", "haziran", "temmuz", "ağustos", "eylül", "ekim", "kasım", "aralık"],
        "it" => &["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"],
        "de" => &["januar", "februar", "märz", "april", "mai", "juni", "juli", "august", "september", "oktober", "november", "dezember"],
        _ => &["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"],
    }
}

/// Parses `s` with a Java-style pattern (the ones sites use): `yyyy yy MMMM MMM MM M dd d HH H hh h mm ss a`,
/// quoted `'literals'`, other characters matched as they are. Case-insensitive, month names by `lang`
/// (English names are always accepted too). `None` when it does not fit.
pub fn parse_date(s: &str, pattern: &str, lang: &str) -> Option<i64> {
    let s: Vec<char> = s.trim().chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let (mut i, mut j) = (0, 0);
    let (mut y, mut mo, mut d, mut h, mut mi, mut sec) = (1970i64, 1i64, 1i64, 0i64, 0i64, 0i64);
    let mut pm: Option<bool> = None;

    let digits = |i: &mut usize, min: usize, max: usize| -> Option<i64> {
        let start = *i;
        while *i < s.len() && *i - start < max && s[*i].is_ascii_digit() {
            *i += 1;
        }
        if *i - start < min {
            return None;
        }
        s[start..*i].iter().collect::<String>().parse().ok()
    };

    while j < p.len() {
        let c = p[j];
        let run = p[j..].iter().take_while(|&&x| x == c).count();
        match c {
            '\'' => {
                // quoted literal; '' is one quote
                let mut k = j + 1;
                let mut lit = String::new();
                while k < p.len() && p[k] != '\'' {
                    lit.push(p[k]);
                    k += 1;
                }
                if lit.is_empty() {
                    lit.push('\'');
                }
                for ch in lit.chars() {
                    if s.get(i).map(|x| x.to_ascii_lowercase()) != Some(ch.to_ascii_lowercase()) {
                        return None;
                    }
                    i += 1;
                }
                j = k + 1;
                continue;
            }
            'y' => {
                y = digits(&mut i, 2, if run >= 4 { 4 } else { 2 })?;
                if run < 4 && y < 100 {
                    y += 2000;
                }
            }
            'M' if run >= 3 => {
                let rest: String = s[i..].iter().collect::<String>().to_lowercase();
                let mut found = None;
                'outer: for names in [month_names(lang), month_names("en")] {
                    // the full name first, so "mar" never cuts "março" short
                    for (idx, name) in names.iter().enumerate() {
                        if rest.starts_with(name) {
                            found = Some((idx, name.chars().count()));
                            break 'outer;
                        }
                    }
                    for (idx, name) in names.iter().enumerate() {
                        let short: String = name.chars().take(3).collect();
                        if rest.starts_with(&short) {
                            found = Some((idx, if rest.chars().nth(3) == Some('.') { 4 } else { 3 }));
                            break 'outer;
                        }
                    }
                }
                let (idx, len) = found?;
                mo = idx as i64 + 1;
                i += len;
            }
            'M' => mo = digits(&mut i, 1, 2)?,
            'd' => d = digits(&mut i, 1, 2)?,
            'H' | 'h' | 'K' | 'k' => h = digits(&mut i, 1, 2)?,
            'm' => mi = digits(&mut i, 1, 2)?,
            's' => sec = digits(&mut i, 1, 2)?,
            'a' => {
                let w: String = s[i..].iter().take(2).collect::<String>().to_lowercase();
                pm = match w.as_str() {
                    "am" => Some(false),
                    "pm" => Some(true),
                    _ => return None,
                };
                i += 2;
            }
            ch => {
                if s.get(i).map(|x| x.to_ascii_lowercase()) != Some(ch.to_ascii_lowercase()) {
                    return None;
                }
                i += 1;
                j += 1;
                continue;
            }
        }
        j += run;
    }
    if let Some(is_pm) = pm {
        h = h % 12 + if is_pm { 12 } else { 0 };
    }
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    Some(millis(y, mo, d, h, mi, sec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_dates() {
        assert_eq!(parse_iso("1970-01-01T00:00:00+00:00"), Some(0));
        assert_eq!(parse_iso("2000-03-01T00:00:00Z"), Some(951_868_800_000));
        assert_eq!(parse_iso("2000-03-01T02:00:00+02:00"), Some(951_868_800_000));
        assert_eq!(parse_iso("2000-03-01T02:00:00.123+0200"), Some(951_868_800_000));
        assert_eq!(parse_iso("2000-03-01"), Some(951_868_800_000));
        assert_eq!(parse_iso("garbage"), None);
        assert_eq!(parse_iso("2000-13-01"), None);
    }

    #[test]
    fn calendar_arithmetic() {
        let t = millis(2024, 3, 31, 10, 20, 0);
        assert_eq!(civil_from_days(days_from_civil(2024, 3, 31)), (2024, 3, 31));
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(minus(t, 2, Unit::Day), millis(2024, 3, 29, 10, 20, 0));
        assert_eq!(minus(t, 1, Unit::Week), millis(2024, 3, 24, 10, 20, 0));
        assert_eq!(minus(t, 1, Unit::Month), millis(2024, 2, 29, 10, 20, 0), "March 31 minus a month is the end of February");
        assert_eq!(minus(t, 13, Unit::Month), millis(2023, 2, 28, 10, 20, 0));
        assert_eq!(minus(t, 1, Unit::Year), millis(2023, 3, 31, 10, 20, 0));
        assert_eq!(minus(t, 90, Unit::Minute), millis(2024, 3, 31, 8, 50, 0));
    }

    #[test]
    fn site_formats() {
        assert_eq!(parse_date("March 1, 2000", "MMMM d, yyyy", "en"), Some(951_868_800_000));
        assert_eq!(parse_date("march 01, 2000", "MMMM dd, yyyy", "en"), Some(951_868_800_000));
        assert_eq!(parse_date("Mar 1, 2000", "MMM d, yyyy", "en"), Some(951_868_800_000));
        assert_eq!(parse_date("01/03/2000", "dd/MM/yyyy", "en"), Some(951_868_800_000));
        assert_eq!(parse_date("2000-03-01 10:30:15", "yyyy-MM-dd HH:mm:ss", "en"), Some(951_868_800_000 + (10 * 3600 + 30 * 60 + 15) * 1000));
        assert_eq!(parse_date("1 de março de 2000", "d 'de' MMMM 'de' yyyy", "pt-BR"), Some(951_868_800_000));
        assert_eq!(parse_date("3:05 pm", "h:mm a", "en"), Some((15 * 3600 + 5 * 60) * 1000));
        assert_eq!(parse_date("not a date", "MMMM d, yyyy", "en"), None);
        assert_eq!(parse_date("March 32, 2000", "MMMM d, yyyy", "en"), None);
    }
}
