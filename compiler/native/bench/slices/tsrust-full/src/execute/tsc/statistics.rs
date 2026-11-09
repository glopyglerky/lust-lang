//! Go: execute/tsc/statistics.go, the `--diagnostics` and
//! `--extendedDiagnostics` table.

use std::time::Duration;

use crate::prelude::*;

// PORT: testing (`report_to`)
use super::compile::{};

struct TableRow {
    name: String,
    value: String,
}

#[derive(Default)]
struct Table {
    rows: Vec<TableRow>,
}

impl Table {
    // Go: execute/tsc/statistics.go:24 add
    // PORT: Go `add` takes `any` and formats a `time.Duration` with
    // `formatDuration`. Durations use `add_duration` here.


    // Go: execute/tsc/statistics.go:31 print
    // One write for each row, as Go's `fmt.Fprintf`.
}

// Go: execute/tsc/statistics.go:44 formatDuration
fn format_duration(d: Duration) -> String {
    format!("{:.3}s", d.as_secs_f64())
}

// PORT: Go execute/tsc/statistics.go:48 `identifierCount` has no caller
// (`statisticsFromProgram` uses `Program.IdentifierCount`), so it is not
// ported.

// Go: execute/tsc/statistics.go:56 Statistics

/// Go `runtime.MemStats`, the two fields that `statisticsFromProgram` reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemStats {
    /// Go `Alloc`: bytes of live heap objects.
    pub alloc: u64,
    /// Go `Mallocs`: count of heap objects allocated.
    pub mallocs: u64,
}


// Go: execute/tsc/statistics.go:72 statisticsFromProgram
// PORT: Go reads the program and the compile times from `EmitInput`. The
// program is the current program (`prog()`) here.


#[cfg(test)]
mod tests {
    use super::*;

    // Go pads names left-aligned to the widest name plus the colon, and
    // values right-aligned to the widest value.
    #[test]
    fn table_print_matches_go_widths() {
        let mut table = Table::default();
        table.add("Files", 750);
        table.add("Memory used", "165135K");
        table.add_duration("Changes compute time", Duration::from_micros(33_400));
        let out = Rc::new(RefCell::new(Vec::<u8>::new()));
        table.print(&(out.clone() as Writer));
        assert_eq!(
            String::from_utf8(out.take()).unwrap(),
            "Files:                    750\n\
             Memory used:          165135K\n\
             Changes compute time:  0.033s\n"
        );
    }

    // Go `table.print` writes each row with one `fmt.Fprintf` on the
    // unbuffered `os.Stdout`. strace of the pin N oracle with
    // `--diagnostics` on a one-file project: these 14 writes of fd 1. The
    // system writer writes each write of the table at once
    // (`stdio::CliStdout`), so `report` must give one write per row.
    // The input is the raw statistics, not the expected text: the test
    // also checks the rows that `report` picks, `Memory used` in KiB
    // (bytes / 1024, truncated), the rounding of each duration to ms, and
    // that a duration under 0.5 ms that is not zero (`Emit time`) keeps
    // its row.
    #[test]
    fn table_print_writes_go_pieces() {
        let go_writes = [
            "Files:              91\n",
            "Lines:           58774\n",
            "Identifiers:     49870\n",
            "Symbols:         59387\n",
            "Types:           34675\n",
            "Instantiations:  33953\n",
            "Memory used:    59443K\n",
            "Memory allocs:  271214\n",
            "Config time:    0.004s\n",
            "Parse time:     0.077s\n",
            "Bind time:      0.041s\n",
            "Check time:     0.387s\n",
            "Emit time:      0.000s\n",
            "Total time:     0.514s\n",
        ];
        let statistics = Statistics {
            files: 91,
            lines: 58774,
            identifiers: 49870,
            symbols: 59387,
            types: 34675,
            instantiations: 33953,
            memory_used: 59443 * 1024 + 1000,
            memory_allocs: 271_214,
            compile_times: Some(CompileTimes {
                config_time: Duration::from_micros(4_400),
                parse_time: Duration::from_micros(76_600),
                bind_time: Duration::from_micros(41_200),
                check_time: Duration::from_micros(386_700),
                emit_time: Duration::from_micros(200),
                total_time: Duration::from_micros(514_300),
                ..CompileTimes::default()
            }),
            ..Statistics::default()
        };
        /// Keeps each write as one piece, an empty one too, as the system
        /// writer writes each one (`GoOutput`).
        #[derive(Default)]
        struct Writes(Vec<String>);
        impl std::io::Write for Writes {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.write_all(buf)?;
                Ok(buf.len())
            }
            fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                self.0.push(String::from_utf8(buf.to_vec()).unwrap());
                Ok(())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let writes = Rc::new(RefCell::new(Writes::default()));
        statistics.report(&(writes.clone() as Writer));
        assert_eq!(writes.take().0, go_writes);
    }

    // The content mapper rows (tsgo#4712): identities in sorted order, and
    // only the operations that ran. Go's map order is random, so the sort
    // is what makes the rows stable.
    #[test]
    fn content_mapper_rows_are_sorted_and_skip_unused_operations() {
        use crate::contentmapper::{MapperTimings, OperationTiming};
        let ran = |millis| OperationTiming {
            count: 1,
            duration: Duration::from_millis(millis),
        };
        let mut compile_times = CompileTimes::default();
        compile_times.content_mapper_times.request_wait = Duration::from_millis(5);
        compile_times.content_mapper_times.mappers.insert(
            "z-mapper".to_string(),
            MapperTimings {
                transform: ran(20),
                ..MapperTimings::default()
            },
        );
        compile_times.content_mapper_times.mappers.insert(
            "a-mapper".to_string(),
            MapperTimings {
                spawn: ran(1),
                initialize: ran(2),
                open_project: ran(3),
                ..MapperTimings::default()
            },
        );
        let statistics = Statistics {
            compile_times: Some(compile_times),
            ..Statistics::default()
        };
        let mut table = Table::default();
        statistics.add_content_mapper_statistics(&mut table, "");
        let rows: Vec<(&str, &str)> = table
            .rows
            .iter()
            .map(|row| (row.name.as_str(), row.value.as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                ("Content mapper request wait time", "0.005s"),
                ("a-mapper initialization time", "0.003s"),
                ("a-mapper openProject time", "0.003s"),
                ("z-mapper transform time", "0.020s"),
            ]
        );
    }
}
