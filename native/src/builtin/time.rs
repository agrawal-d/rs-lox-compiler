use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::Value;

callable_struct!(Now, "now", 0, "now()
Returns current Unix timestamp in seconds.
Arguments: None.
Returns: Number Unix timestamp in seconds.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => Value::Number(dur.as_secs_f64()),
        Err(_) => Value::Number(0.0),
    }
});

callable_struct!(NowMs, "now_ms", 0, "now_ms()
Returns current Unix timestamp in milliseconds.
Arguments: None.
Returns: Number Unix timestamp in milliseconds.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => Value::Number(dur.as_millis() as f64),
        Err(_) => Value::Number(0.0),
    }
});

callable_struct!(Format, "format", 0, "format([timestamp], [format_str])
Formats a Unix timestamp in seconds into a date string.
Arguments:
  timestamp: (Optional) Number Unix timestamp in seconds (default: current time).
  format_str: (Optional) Format specifiers (default: \"%Y-%m-%d %H:%M:%S\").
Returns: String formatted date.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let ts_sec = if !args.is_empty() {
        match &args[0] {
            Value::Number(n) => *n,
            _ => SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0),
        }
    } else {
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
    };

    let fmt_spec = if args.len() > 1 {
        match &args[1] {
            Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
            _ => "%Y-%m-%d %H:%M:%S",
        }
    } else {
        "%Y-%m-%d %H:%M:%S"
    };

    let total_secs = ts_sec as i64;
    let days = total_secs / 86400;
    let seconds_into_day = total_secs % 86400;

    let hours = seconds_into_day / 3600;
    let minutes = (seconds_into_day % 3600) / 60;
    let seconds = seconds_into_day % 60;

    // Epoch (1970-01-01) calculation helper
    let (year, month, day) = epoch_days_to_date(days);

    let mut result = fmt_spec.to_string();
    result = result.replace("%Y", &format!("{:04}", year));
    result = result.replace("%m", &format!("{:02}", month));
    result = result.replace("%d", &format!("{:02}", day));
    result = result.replace("%H", &format!("{:02}", hours));
    result = result.replace("%M", &format!("{:02}", minutes));
    result = result.replace("%S", &format!("{:02}", seconds));

    Value::Str(interner.intern(&result))
});

fn epoch_days_to_date(mut days: i64) -> (i64, i64, i64) {
    days += 719468;
    let era = (if days >= 0 { days } else { days - 146096 }) / 146097;
    let doe = (days - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = ((5 * doy + 2) / 153) as i64;
    let d = (doy - (153 * (mp as u64) + 2) / 5 + 1) as i64;
    let m = mp + (if mp < 10 { 3 } else { -9 });
    let year = y + (if m <= 2 { 1 } else { 0 });
    (year, m, d)
}

callable_struct!(Sleep, "sleep", 1, "sleep(ms)
Sleeps for the specified duration in milliseconds.
Arguments:
  ms: Number of milliseconds to sleep.
Returns: Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Number(ms) => {
            if *ms > 0.0 {
                std::thread::sleep(Duration::from_millis(*ms as u64));
            }
            Value::Nil
        }
        _ => { set_global_error(interner, globals, "Expected number of milliseconds for time.sleep"); Value::Nil }
    }
});

callable_struct!(Elapsed, "elapsed", 1, "elapsed(start_time_ms)
Calculates elapsed milliseconds since start_time_ms.
Arguments:
  start_time_ms: Number starting timestamp in milliseconds.
Returns: Number elapsed milliseconds.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let start_ms = match &args[0] {
        Value::Number(n) => *n,
        _ => { set_global_error(interner, globals, "Expected start timestamp in milliseconds for time.elapsed"); return Value::Nil; }
    };

    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0);
    Value::Number((now_ms - start_ms).max(0.0))
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("now", Rc::new(Now)),
        ("now_ms", Rc::new(NowMs)),
        ("format", Rc::new(Format)),
        ("sleep", Rc::new(Sleep)),
        ("elapsed", Rc::new(Elapsed)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
