// Standard library imports
use std::{
    fs::File,
    io::{self, Result},
    thread,
    time::{Duration, Instant},
};

// External crate imports
use crossterm::{
    cursor::MoveToColumn,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

// Internal crate imports
use crate::utils::{adjust_unit, pr_time_str};

enum State {
    Inc,
    Dec,
    Search,
    Steady,
}
pub struct Flow {
    // Total number of bytes to be processed.
    total_size: u64,
    // Total number of bytes already processed.
    total_processed: u64,
    //  If true, show progress.
    progress: bool,
    // Size of each block to be written (in bytes).
    block_size: i32,
    // Delay between writes (in milliseconds).
    delay_ms: u64,
    // Increment to apply to @blocks_per_delay.
    step: i64,
    // Blocks to process before measurement.
    blocks_per_delay: i64,
    // Maximum processing rate in bytes per second.
    max_process_rate: f64,
    //  Number of blocks processed in the last measurement.
    measured_blocks: u64,
    //  Measured time.
    measured_time_ms: u64,
    // Current state of the flow.
    state: State,

    processed_blocks: i64,
    acc_delay_us: u64,

    bpd_low: i64,
    bpd_high: i64,

    measure_start_time: Instant,
    last_inst_bps: f64,
    last_report_time: Instant,
}

impl Flow {
    pub fn new(total_size: u64, max_process_rate: i64, progress: bool) -> Flow {
        Flow {
            total_size,
            total_processed: 0,
            progress,
            block_size: 512,     // Default block size of 512 bytes
            delay_ms: 1000,      // 1s
            blocks_per_delay: 1, // 512 B/s
            max_process_rate: if max_process_rate <= 0 {
                f64::MAX
            } else {
                (max_process_rate * 1024) as f64
            }, // Convert Kb/s to bytes/s
            measured_blocks: 0,
            measured_time_ms: 0,
            processed_blocks: 0,
            acc_delay_us: 0,
            measure_start_time: Instant::now(),

            state: State::Inc,
            step: 1,

            bpd_low: 0,
            bpd_high: 0,
            last_inst_bps: 0.0,
            last_report_time: Instant::now(),
        }
    }

    pub fn start_measurement(&mut self) {
        if self.progress && self.has_enough_measurements() {
            self.report_progress();
        }

        self.measure_start_time = Instant::now();
    }

    pub fn report_progress(&mut self) {
        let inst_bps = if self.last_inst_bps.is_finite() {
            self.last_inst_bps
        } else {
            0.0
        };
        // let inst_speed =
        //     (self.blocks_per_delay * self.block_size as i64) as f64 * 1000.0 / self.delay_ms as f64;

        let avg_bps = if self.has_enough_measurements() {
            self.get_avg_speed()
        } else {
            inst_bps
        };

        let (inst_speed, unit) = adjust_unit(avg_bps);
        if self.total_size < self.total_processed {
            self.total_size = self.total_processed;
        }
        let percent = (self.total_processed as f64 * 100.0) / self.total_size as f64;

        let mut progress_str = format!("{:.2}% -- {:.2} {}/s", percent, inst_speed, unit);

        if self.has_enough_measurements() {
            let eta = (self.total_size - self.total_processed) as f64 / self.get_avg_speed();
            progress_str.push_str(" -- remaining time: ");
            progress_str.push_str(&pr_time_str(eta));
        }

        execute!(
            io::stdout(),
            Clear(ClearType::CurrentLine),
            MoveToColumn(0),
            Print(progress_str)
        )
        .unwrap();
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }

    pub fn get_avg_speed(&mut self) -> f64 {
        ((self.measured_blocks * self.block_size as u64 * 1000) / self.measured_time_ms) as f64
    }

    pub fn has_enough_measurements(&self) -> bool {
        self.measured_time_ms > self.delay_ms
    }

    pub fn get_avg_speed_given_time(&self, total_time_ms: u64) -> f64 {
        if total_time_ms > 0 {
            ((self.processed_blocks * self.block_size as i64 * 1000) / total_time_ms as i64) as f64
        } else {
            0.0
        }
    }

    pub fn get_remaining_chunk_size(&self) -> u64 {
        assert!(self.blocks_per_delay as i64 > self.processed_blocks);
        let left_blocks = (self.blocks_per_delay - self.processed_blocks as i64) as u64;
        left_blocks * self.block_size as u64
    }

    const REPORT_INTERVAL_SECS: u64 = 1;

    pub fn measure(&mut self, file: &File, processed: i64) -> Result<()> {
        assert!(processed % self.block_size as i64 == 0);

        self.processed_blocks += processed / self.block_size as i64;
        self.total_processed += processed as u64;

        if self.processed_blocks < self.blocks_per_delay {
            return Ok(());
        }

        assert!(self.processed_blocks == self.blocks_per_delay);
        self.flush_chunk(file)?;
        let end_time = Instant::now();
        let mut delay: u64 = (end_time - self.measure_start_time
            + Duration::from_micros(self.acc_delay_us))
        .as_millis() as u64;
        if delay == 0 {
            delay = 1;
        }

        // Instantaneous speed in bytes per second.
        let bytes_k: f64 = (self.blocks_per_delay * self.block_size as i64 * 1000) as f64;
        let mut inst_speed = bytes_k / delay as f64;

        if delay < self.delay_ms && inst_speed > self.max_process_rate {
            let mut wait_ms =
                ((bytes_k - delay as f64 * self.max_process_rate) / self.max_process_rate).round();

            if wait_ms < 0.0 {
                wait_ms = (self.delay_ms - delay) as f64;
            } else if (delay as f64 + wait_ms) < self.delay_ms as f64 {
                //   wait_ms is not the largest possible value, so
                //  force the flow algorithm to keep increasing it.
                //  Otherwise, the delay to print progress may be
                //  too small.

                wait_ms += 1.0;
            }

            if wait_ms > 0.0 && self.max_process_rate != f64::MAX {
                // Slow down.
                thread::sleep(Duration::from_millis(wait_ms as u64));

                //Adjust measurements.
                delay += wait_ms as u64;
                inst_speed = bytes_k / delay as f64;
            }
        }

        self.measured_blocks += self.processed_blocks as u64;
        self.measured_time_ms += delay;

        self.last_inst_bps = inst_speed;

        self.adjust_state(inst_speed, delay);

        if self.progress && self.last_report_time.elapsed().as_secs() >= Self::REPORT_INTERVAL_SECS
        {
            self.report_progress();
        }

        self.processed_blocks = 0;
        self.acc_delay_us = 0;

        self.measure_start_time = Instant::now();

        Ok(())
    }

    pub fn pr_avg_speed(&mut self) {
        let avg_speed = self.get_avg_speed();
        let (size, unit) = adjust_unit(avg_speed);
        println!("Average speed: {:.2} {}/s", size, unit);
    }

    pub fn end_measurement(&mut self, file: &File) -> Result<()> {
        execute!(io::stdout(), MoveToColumn(0), Clear(ClearType::CurrentLine),).unwrap();

        if self.processed_blocks <= 0 {
            return Ok(());
        }

        self.flush_chunk(file)?;

        let end_time = Instant::now();
        self.acc_delay_us += diff_in_us(self.measure_start_time, end_time);

        Ok(())
    }

    fn dec_step(&mut self) {
        use State::*;
        if self.blocks_per_delay - self.step > 0 {
            self.blocks_per_delay -= self.step;
            self.step *= 2;
        } else {
            self.bpd_high = self.blocks_per_delay.max(1);
            self.bpd_low = (self.blocks_per_delay - self.step / 2).max(1);
            self.blocks_per_delay = 1;
            self.state = Search;
        }
    }

    fn inc_step(&mut self) {
        self.blocks_per_delay += self.step;
        self.step *= 2;
    }

    fn move_to_search(&mut self, low: i64, high: i64) {
        use State::*;
        assert!(low > 0);
        assert!(high >= low);
        self.blocks_per_delay = (low + high) / 2;
        if high - low <= 3 {
            self.state = Steady;
            return;
        }

        self.bpd_low = low;
        self.bpd_high = high;

        self.state = Search;
    }

    fn is_rate_above(&self, delay: u64, inst_speed: f64) -> bool {
        delay > self.delay_ms || inst_speed > self.max_process_rate
    }

    fn is_rate_below(&self, delay: u64, inst_speed: f64) -> bool {
        delay <= self.delay_ms && inst_speed < self.max_process_rate
    }

    fn adjust_state(&mut self, inst_speed: f64, delay: u64) {
        use State::*;
        let above = self.is_rate_above(delay, inst_speed);
        let below = self.is_rate_below(delay, inst_speed);

        match self.state {
            Inc => {
                if above {
                    self.move_to_search(
                        self.blocks_per_delay - self.step / 2,
                        self.blocks_per_delay,
                    )
                } else if below {
                    self.inc_step();
                } else {
                    self.state = Steady;
                }
            }
            Dec => {
                if above {
                    self.dec_step();
                } else if below {
                    self.move_to_search(
                        self.blocks_per_delay,
                        self.blocks_per_delay + self.step / 2,
                    )
                } else {
                    self.state = Steady;
                }
            }
            Search => {
                if self.bpd_high - self.bpd_low <= 3 {
                    self.state = Steady;
                    return;
                }

                if above {
                    self.bpd_high = self.blocks_per_delay;
                    self.blocks_per_delay = (self.bpd_low + self.bpd_high) / 2;
                } else if below {
                    self.bpd_low = self.blocks_per_delay;
                    self.blocks_per_delay = (self.bpd_low + self.bpd_high) / 2;
                } else {
                    self.state = Steady;
                }
            }
            Steady => {
                self.step = 1;
                if delay <= self.delay_ms as u64 {
                    if inst_speed < self.max_process_rate {
                        self.state = Inc;
                        self.inc_step();
                    } else if self.blocks_per_delay > 1 {
                        self.state = Dec;
                        self.dec_step();
                    }
                } else if self.blocks_per_delay > 1 {
                    self.state = Dec;
                    self.dec_step();
                }
            }
        }
    }

    pub fn flush_chunk(&self, file: &File) -> Result<()> {
        file.sync_data()?;

        Ok(())
    }
}

#[inline]
pub fn diff_in_us(t1: Instant, t2: Instant) -> u64 {
    t2.duration_since(t1).as_micros() as u64
}

const DEFAULT_BUF_SIZE: usize = 2 * 1024 * 1024; // 2MB
pub struct DynamicBuffer {
    buf: Vec<u8>,
    max_buf: bool,
}

impl DynamicBuffer {
    pub fn new() -> Self {
        Self {
            buf: vec![0u8; DEFAULT_BUF_SIZE],
            max_buf: false,
        }
    }

    pub fn get_buf(&mut self, size: usize) -> &mut [u8] {
        if size <= self.get_len() || self.max_buf {
            return &mut self.buf;
        }

        if self.buf.len() < size {
            self.buf.resize(size, 0);
            self.max_buf = true;
        }
        self.max_buf = true;
        &mut self.buf[..size]
    }

    pub fn get_len(&self) -> usize {
        self.buf.len()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }
}
